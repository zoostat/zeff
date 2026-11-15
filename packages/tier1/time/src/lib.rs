//! Package `time` (tier 1): dates and times (Engine and Project Specification, section 8.2;
//! Language Reference, section 3.8, "Time"). At this release the parsing functions `date` and
//! `datetime` and the date parts `year`, `month`, `day`, `weekday`, `quarter` and `week`
//! (release 0.6.0, step 3); the arithmetic `add_days` and `add_months` and the clock's `today`
//! and `now` (step 4); `time_between`, which took the place of `days_between` and
//! `months_between` (release 0.8.0, step 8d). Every function is a scalar
//! function that applies row by row to a column (section 3.7, "Columns and scalars"), and every
//! one computes in the dates kernel, in integer arithmetic; the clock is the run's, given by the
//! command line or fixed by a test, never read by the engine itself.

use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_dates::{
    DateFormat, EpochUnit, FixedUnit, civil_from_days, days_of_micros, format_iso_date,
    format_iso_datetime, iso_week, micros_of_days, parse_iso_date, parse_iso_datetime, quarter_of,
    weekday_of,
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
    // `epoch:` reads a number as Unix time (open decision #96): seconds or milliseconds since
    // 1970-01-01 00:00:00 UTC. Text is read without it, and a number only with it, so that a
    // number such as 20261016 is never taken for a count of seconds by mistake.
    let epoch = match args.get(2) {
        Some(Value::Word(w)) if w == "seconds" => Some(EpochUnit::Seconds),
        Some(Value::Word(w)) if w == "milliseconds" => Some(EpochUnit::Milliseconds),
        _ => None,
    };
    if let Some(unit) = epoch {
        if format.is_some() {
            return Err(fail(
                Diagnostic::error("Z0410", "`datetime` takes `format:` or `epoch:`, not both")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "`epoch:` beside `format:`")
                    .help("`format:` says how text is written and `epoch:` what a number counts; keep the one that fits `s`"),
            ));
        }
        let micros = match args.get(0) {
            None | Some(Value::Na) => return Ok(Value::Na),
            Some(Value::Int(n)) => zeff_dates::micros_of_epoch(*n, unit),
            Some(Value::Float(x)) if x.is_nan() => return Ok(Value::Na),
            Some(Value::Float(x)) => zeff_dates::micros_of_epoch_float(*x, unit),
            Some(Value::Str(_)) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        "`epoch:` reads a number of Unix time, but `s` is text",
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(0), "text")
                    .help("leave `epoch:` out to read text as a date and time, or make the text a number first with `to(s, Int)`"),
                ));
            }
            Some(_) => return Err(internal(ctx, "a value that is not a number")),
        };
        return Ok(match micros {
            Some(micros) => Value::DateTime(micros),
            None => {
                outside_the_calendar(ctx);
                Value::Na
            }
        });
    }
    let text = match args.get(0) {
        None | Some(Value::Na) => return Ok(Value::Na),
        Some(Value::Str(s)) => s.trim(),
        Some(number @ (Value::Int(_) | Value::Float(_))) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`datetime` reads text, but `s` is {}",
                        number.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "a number")
                .help("a number is Unix time only when the call says so: `epoch: seconds` or `epoch: milliseconds`"),
            ));
        }
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

/// A part of the time of day of a `DateTime`, as an Int; `NA` for `NA`.
fn clock_part(
    ctx: &CallContext<'_>,
    args: &Args,
    f: fn((u32, u32, u32)) -> u32,
) -> Result<Value, Fail> {
    match args.get(0) {
        None | Some(Value::Na) => Ok(Value::Na),
        Some(Value::DateTime(micros)) => Ok(Value::Int(i64::from(f(zeff_dates::clock_of_micros(
            *micros,
        ))))),
        Some(_) => Err(internal(ctx, "a value that is not a date and time")),
    }
}

/// `hour(dt)`: the hour of the day, 0 to 23.
fn hour(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    clock_part(ctx, &args, |(h, _, _)| h)
}

/// `minute(dt)`: the minute of the hour, 0 to 59.
fn minute(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    clock_part(ctx, &args, |(_, m, _)| m)
}

/// `second(dt)`: the whole second of the minute, 0 to 59, a fraction dropped.
fn second(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    clock_part(ctx, &args, |(_, _, s)| s)
}

/// W0203 for a date moved outside the calendar.
fn outside_the_calendar(ctx: &mut CallContext<'_>) {
    ctx.message(
        Diagnostic::warning("W0203", "a date outside years 1 to 9999 became NA")
            .help("the calendar runs from 0001-01-01 to 9999-12-31 (section 3.8 of the Language Reference)"),
    );
}

/// The Int argument at `at`, or `None` for `NA`.
fn int_at(ctx: &CallContext<'_>, args: &Args, at: usize) -> Result<Option<i64>, Fail> {
    match args.get(at) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Int(n)) => Ok(Some(*n)),
        Some(_) => Err(internal(ctx, "a count that is not an Int")),
    }
}

/// `add_days(d, n)`: the date or time `n` days later (earlier for a negative `n`), `NA` outside
/// the calendar.
fn add_days(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(n) = int_at(ctx, &args, 1)? else {
        return Ok(Value::Na);
    };
    Ok(match args.get(0) {
        None | Some(Value::Na) => Value::Na,
        Some(Value::Date(days)) => match zeff_dates::add_days(*days, n) {
            Some(moved) => Value::Date(moved),
            None => {
                outside_the_calendar(ctx);
                Value::Na
            }
        },
        Some(Value::DateTime(micros)) => match zeff_dates::add_days(days_of_micros(*micros), n) {
            Some(day) => Value::DateTime(
                micros_of_days(day) + (micros - micros_of_days(days_of_micros(*micros))),
            ),
            None => {
                outside_the_calendar(ctx);
                Value::Na
            }
        },
        Some(_) => return Err(internal(ctx, "a value that is not a date or a time")),
    })
}

/// `add_months(d, n)`: the date or time `n` months later, the day clamped to the last day of
/// the target month (section 3.8, "Month arithmetic"), `NA` outside the calendar.
fn add_months(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(n) = int_at(ctx, &args, 1)? else {
        return Ok(Value::Na);
    };
    Ok(match args.get(0) {
        None | Some(Value::Na) => Value::Na,
        Some(Value::Date(days)) => match zeff_dates::add_months(*days, n) {
            Some(moved) => Value::Date(moved),
            None => {
                outside_the_calendar(ctx);
                Value::Na
            }
        },
        Some(Value::DateTime(micros)) => match zeff_dates::add_months(days_of_micros(*micros), n) {
            Some(day) => Value::DateTime(
                micros_of_days(day) + (micros - micros_of_days(days_of_micros(*micros))),
            ),
            None => {
                outside_the_calendar(ctx);
                Value::Na
            }
        },
        Some(_) => return Err(internal(ctx, "a value that is not a date or a time")),
    })
}

/// The unit a difference is counted in: the word after `unit:`, `day` unless given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Fixed(FixedUnit),
    Month,
    Year,
}

impl Unit {
    fn of(word: Option<&Value>) -> Unit {
        match word {
            Some(Value::Word(w)) => match w.as_str() {
                "second" => Unit::Fixed(FixedUnit::Second),
                "minute" => Unit::Fixed(FixedUnit::Minute),
                "hour" => Unit::Fixed(FixedUnit::Hour),
                "week" => Unit::Fixed(FixedUnit::Week),
                "month" => Unit::Month,
                "year" => Unit::Year,
                _ => Unit::Fixed(FixedUnit::Day),
            },
            _ => Unit::Fixed(FixedUnit::Day),
        }
    }

    fn word(self) -> &'static str {
        match self {
            Unit::Fixed(FixedUnit::Second) => "second",
            Unit::Fixed(FixedUnit::Minute) => "minute",
            Unit::Fixed(FixedUnit::Hour) => "hour",
            Unit::Fixed(FixedUnit::Day) => "day",
            Unit::Fixed(FixedUnit::Week) => "week",
            Unit::Month => "month",
            Unit::Year => "year",
        }
    }

    /// Whether the unit is shorter than a day, and so needs a time of day on both sides.
    fn of_the_clock(self) -> bool {
        matches!(
            self,
            Unit::Fixed(FixedUnit::Second | FixedUnit::Minute | FixedUnit::Hour)
        )
    }
}

/// `time_between(start, end, unit:)`: the whole units from `start` to `end`, truncated towards
/// zero, as an Int (section 3.8, "Differences"; open decision #96). Between two dates the days
/// are counted; between two times the instants, so 23 hours is no day and a month less an hour
/// no month. Months and years clamp as `add_months` does, a year being twelve such months.
fn time_between(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let unit = Unit::of(args.get(2));
    Ok(match (args.get(0), args.get(1)) {
        (None | Some(Value::Na), _) | (_, None | Some(Value::Na)) => Value::Na,
        (Some(Value::Date(a)), Some(Value::Date(b))) => {
            let days = i64::from(*b) - i64::from(*a);
            match unit {
                Unit::Fixed(FixedUnit::Day) => Value::Int(days),
                Unit::Fixed(FixedUnit::Week) => Value::Int(days / 7),
                Unit::Month => Value::Int(zeff_dates::months_between(*a, *b)),
                Unit::Year => Value::Int(zeff_dates::years_between(*a, *b)),
                Unit::Fixed(_) => {
                    // A date has no time of day, so no count of hours: midnight would be a
                    // guess, and a guess is never made for the reader.
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`unit: {}` counts between two times, but `start` and `end` are dates",
                                unit.word()
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(2), "a unit of the clock")
                        .help("a Date has no time of day: count with `unit: day`, or give two DateTimes (`to(d, DateTime)` is midnight of the day)"),
                    ));
                }
            }
        }
        (Some(Value::DateTime(a)), Some(Value::DateTime(b))) => Value::Int(match unit {
            Unit::Fixed(fixed) => zeff_dates::whole_units_between(*a, *b, fixed),
            Unit::Month => zeff_dates::months_between_micros(*a, *b),
            Unit::Year => zeff_dates::years_between_micros(*a, *b),
        }),
        (
            Some(a @ (Value::Date(_) | Value::DateTime(_))),
            Some(b @ (Value::Date(_) | Value::DateTime(_))),
        ) => {
            // A date beside a time: the checker admits either for each argument, so the pair is
            // refused here, with the conversion to write.
            let (to_date, to_time) = if matches!(a, Value::Date(_)) {
                ("end", "start")
            } else {
                ("start", "end")
            };
            let help = if unit.of_the_clock() {
                format!(
                    "write `to({to_time}, DateTime)` to count between the instants, midnight standing for the date"
                )
            } else {
                format!(
                    "write `to({to_date}, Date)` to count between the dates, or `to({to_time}, DateTime)` to count between the instants"
                )
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`time_between` counts between two dates or two times, but `start` is {} and `end` is {}",
                        a.type_word(),
                        b.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "not the type of `start`")
                .help(help),
            ));
        }
        _ => {
            return Err(internal(
                ctx,
                "two values that are not both dates or both times",
            ));
        }
    })
}

/// The run's clock, noted at each use (W0703), or an internal error for a run without one.
fn clock(ctx: &mut CallContext<'_>, function: &str) -> Result<i64, Fail> {
    ctx.files.now().ok_or_else(|| {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script (`{function}` was called in a run that has no clock)"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )
    })
}

/// `today()`: the clock's date.
fn today(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    let days = days_of_micros(clock(ctx, "today")?);
    ctx.message(
        Diagnostic::note("W0703", format!("today() read the clock: {}", format_iso_date(days)))
            .help("the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)"),
    );
    Ok(Value::Date(days))
}

/// `now()`: the clock's date and time, the machine's local wall-clock time with no zone.
fn now(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    let micros = clock(ctx, "now")?;
    ctx.message(
        Diagnostic::note("W0703", format!("now() read the clock: {}", format_iso_datetime(micros)))
            .help("the value depends on when the script runs; `zeff run --replay` will reuse it (release 0.11.0)"),
    );
    Ok(Value::DateTime(micros))
}

/// A move of a date: a Date or a DateTime and an Int, giving the type of the first.
fn moved(name: &'static str, help: &'static str, body: zeff_core::runtime::Body) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "time",
            method: false,
            params: vec![
                ParamSpec::required("d", "Date | DateTime"),
                ParamSpec::required("n", "Int"),
            ],
            rest: None,
            returns: "Same",
            diagnostics: vec![DiagnosticDecl::new(
                "W0203",
                "the result lies outside years 1 to 9999, so it is NA",
            )],
            help,
            accuracy: None,
        },
        body,
    }
}

/// A reading of the clock: no parameter, a value noted at each use.
fn clock_reading(
    name: &'static str,
    returns: &'static str,
    help: &'static str,
    body: zeff_core::runtime::Body,
) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "time",
            method: false,
            params: vec![],
            rest: None,
            returns,
            diagnostics: vec![DiagnosticDecl::new(
                "W0703",
                "the value was read from the clock and is noted, since it depends on when the script runs",
            )],
            help,
            accuracy: None,
        },
        body,
    }
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

/// A part of the time of day: `hour`, `minute`, `second`. They take a `DateTime` only, since a
/// `Date` has no time of day and midnight would be a guess.
fn time_part(name: &'static str, help: &'static str, body: zeff_core::runtime::Body) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "time",
            method: false,
            params: vec![ParamSpec::required("dt", "DateTime")],
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
                    ParamSpec::required("s", "Str | Int | Float"),
                    ParamSpec::option("format", "Str"),
                    ParamSpec::option("epoch", "seconds | milliseconds"),
                ],
                rest: None,
                returns: "DateTime",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0203",
                        "the text is not a date and time in ISO 8601 or in the format, or with epoch: the number lies outside years 1 to 9999, so the value is NA",
                    ),
                    DiagnosticDecl::new(
                        "W0702",
                        "the text carried a zone offset, which was applied to reach UTC",
                    ),
                    DiagnosticDecl::new("Z0409", "format: is not a date format"),
                    DiagnosticDecl::new("Z0401", "s is a number without epoch:, or text with it"),
                    DiagnosticDecl::new("Z0410", "format: and epoch: are both given"),
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
        time_part("hour", include_str!("../help/hour.md"), hour),
        time_part("minute", include_str!("../help/minute.md"), minute),
        time_part("second", include_str!("../help/second.md"), second),
        moved("add_days", include_str!("../help/add_days.md"), add_days),
        moved(
            "add_months",
            include_str!("../help/add_months.md"),
            add_months,
        ),
        Function {
            spec: FunctionSpec {
                name: "time_between",
                package: "time",
                method: false,
                params: vec![
                    ParamSpec::required("start", "Date | DateTime"),
                    ParamSpec::required("end", "Date | DateTime"),
                    ParamSpec::option_with_default(
                        "unit",
                        "second | minute | hour | day | week | month | year",
                        "day",
                    ),
                ],
                rest: None,
                returns: "Int",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "start and end are a date and a time, not two of one kind; or unit: is second, minute or hour and they are dates",
                )],
                help: include_str!("../help/time_between.md"),
                accuracy: None,
            },
            body: time_between,
        },
        clock_reading("today", "Date", include_str!("../help/today.md"), today),
        clock_reading("now", "DateTime", include_str!("../help/now.md"), now),
    ]
}
