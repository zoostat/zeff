//! The scales of the two axes (Language Reference, section 7.6, "Scales"): `scale_x` and
//! `scale_y` say the kind of scale of an axis of numbers, linear, log or square root, its
//! limits, its breaks and their labels; of an axis of dates, the kind `date`, they say the
//! limits and the breaks as dates and the pattern the breaks are written by. What they say
//! is held with the plot (`Plot.axes`), and the scene derives the scale from it.
//!
//! The limits are the visible range and nothing more (open decision #118, F5): no row is
//! dropped for them, the marks beyond them are cut off at the edge of the panel and counted in
//! a note, and a layer that summarises its rows uses every one of them. Removing rows is
//! `where`. A value with no place on the scale, at or below zero on a log scale or below zero
//! on a square-root one, is an error naming the rows.

use std::rc::Rc;

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, AxisKind, Plot, PlotAxis, Value};
use zeff_diagnostics::Diagnostic;

use crate::dates;
use crate::grammar::{internal, shown};
use crate::marks::misplaced;
use crate::scene::{Misplaced, Placing, axis_fault, describe, placing_of, scene};

const P: usize = 0;
const KIND: usize = 1;
const LIMITS: usize = 2;
const BREAKS: usize = 3;
const LABELS: usize = 4;

/// `p.scale_x(type: linear | log | sqrt | date = linear, limits: List, breaks: List,
/// labels: List | Str)`.
pub fn scale_x(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    axis(ctx, args, "scale_x", Aesthetic::X)
}

/// `p.scale_y(...)`, with the options of `scale_x`.
pub fn scale_y(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    axis(ctx, args, "scale_y", Aesthetic::Y)
}

/// A number of a list, `None` for `NA`; anything else is not a number.
fn number(value: &Value) -> Result<Option<f64>, ()> {
    match value {
        Value::Na => Ok(None),
        Value::Int(i) => Ok(Some(*i as f64)),
        Value::Float(x) if x.is_finite() => Ok(Some(*x)),
        _ => Err(()),
    }
}

/// A date of a list as its instant on a scale of dates, `None` for `NA`; anything else is
/// not a date.
fn instant(value: &Value) -> Result<Option<f64>, ()> {
    match value {
        Value::Na => Ok(None),
        Value::Date(days) => Ok(Some(f64::from(*days) * dates::DAY as f64)),
        Value::DateTime(micros) => Ok(Some(*micros as f64)),
        _ => Err(()),
    }
}

/// Whether a list holds a date.
fn holds_dates(items: &[Value]) -> bool {
    items
        .iter()
        .any(|item| matches!(item, Value::Date(_) | Value::DateTime(_)))
}

/// What `scale_x` and `scale_y` share.
fn axis(
    ctx: &mut CallContext<'_>,
    args: Args,
    function: &'static str,
    aesthetic: Aesthetic,
) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a scale reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let name = aesthetic.name();
    let kind = match args.get(KIND) {
        None | Some(Value::Na) => AxisKind::Linear,
        Some(Value::Word(word)) => match word.as_str() {
            "linear" => AxisKind::Linear,
            "log" => AxisKind::Log,
            "sqrt" => AxisKind::Sqrt,
            "date" => AxisKind::Date,
            _ => return Err(internal(ctx, "a kind of scale that is none of the four")),
        },
        Some(_) => return Err(internal(ctx, "a kind of scale that is not a word")),
    };
    // An axis of categories has its places and no other scale.
    if let Some((Placing::Categories, variable)) = placing_of(&plot, aesthetic) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "the `{name}` axis holds the categories of `{variable}`, which have their places and no other scale"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the axis is of categories")
            .help(format!(
                "`{function}` sets the scale of an axis of numbers or of dates: its kind, its limits, its breaks and their labels"
            )),
        ));
    }
    // An axis of dates is placed in proportion to the time whatever is said, so a linear
    // scale on it, or no kind at all, is the scale of dates: its limits and breaks are dates.
    let dated = kind == AxisKind::Date
        || (kind == AxisKind::Linear
            && matches!(placing_of(&plot, aesthetic), Some((Placing::Dates, _))));
    let kind = if dated { AxisKind::Date } else { kind };
    let at_kind = if matches!(args.get(KIND), None | Some(Value::Na)) {
        ctx.callee_span
    } else {
        ctx.span_of(KIND)
    };
    // A kind the axis cannot have is said before anything else of the call is read: a
    // scale of dates on numbers, a scale of numbers on dates.
    {
        let mut tried = (*plot).clone();
        tried.axes.retain(|axis| axis.aesthetic != aesthetic);
        tried.axes.push(PlotAxis::of(aesthetic, kind));
        if let Some(fault) = axis_fault(&tried)
            && matches!(fault.what, Misplaced::Dates(_) | Misplaced::Numbers(_))
        {
            return Err(misplaced(ctx, &fault, at_kind));
        }
    }
    // A limit or a break as the axis holds it: a number, or on a scale of dates a date.
    let read = |value: &Value| if dated { instant(value) } else { number(value) };
    let told = |v: f64| if dated { dates::moment(v) } else { shown(v) };
    let na_help = if dated {
        "`NA` in the place of one leaves that end to the dates drawn"
    } else {
        "`NA` leaves an end to the values drawn, as in `limits: [0, NA]`"
    };
    let (things, a_thing, sample_limits, sample_breaks) = if dated {
        (
            "dates",
            "a date",
            format!("{function}(date, limits: [date(\"2024-01-01\"), date(\"2024-12-31\")])"),
            format!("{function}(date, breaks: [date(\"2024-01-01\"), date(\"2024-07-01\")])"),
        )
    } else {
        (
            "numbers",
            "a number",
            format!("{function}(limits: [0, 100])"),
            format!("{function}(breaks: [0, 25, 50, 75, 100])"),
        )
    };
    // Dates given to an axis of numbers are named for what they are.
    let dates_on_numbers = |ctx: &CallContext<'_>, slot: usize, option: &str| {
        fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{option}:` of `{function}` holds dates, but the `{name}` axis is {}",
                    if kind == AxisKind::Linear {
                        "one of numbers".to_string()
                    } else {
                        format!("on a {} scale", kind.name())
                    }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "dates on an axis of numbers")
            .help(format!("an axis of dates is one a Date or a DateTime variable is mapped to; `{function}(date, ...)` says the kind outright")),
        )
    };
    let holds = |v: f64| match kind {
        AxisKind::Log => v > 0.0,
        AxisKind::Sqrt => v >= 0.0,
        AxisKind::Linear | AxisKind::Date => true,
    };
    let place = match kind {
        AxisKind::Log => "a log scale holds the values above 0",
        AxisKind::Sqrt => "a square-root scale holds the values of 0 or more",
        AxisKind::Linear | AxisKind::Date => "",
    };
    // The limits: a list of two, the lower and the upper, either of which may be `NA` for
    // the end the values drawn decide.
    let limits = match args.get(LIMITS) {
        None | Some(Value::Na) => (None, None),
        Some(Value::List(items)) => {
            let refused = |ctx: &CallContext<'_>, why: String| {
                fail(
                    Diagnostic::error("Z0401", why)
                        .in_file(ctx.script)
                        .primary(ctx.span_of(LIMITS), "not two limits")
                        .help(format!("write the lower limit and the upper, as in `{sample_limits}`; {na_help}")),
                )
            };
            if !dated && holds_dates(items) {
                return Err(dates_on_numbers(ctx, LIMITS, "limits"));
            }
            let [low, high] = items.as_slice() else {
                return Err(refused(
                    ctx,
                    format!(
                        "`limits:` of `{function}` is a list of two {things}, the lower limit and the upper, but this list has {}",
                        items.len()
                    ),
                ));
            };
            let (Ok(low), Ok(high)) = (read(low), read(high)) else {
                return Err(refused(
                    ctx,
                    if dated {
                        format!(
                            "`limits:` of `{function}` on an axis of dates is a list of two dates, the first shown and the last"
                        )
                    } else {
                        format!(
                            "`limits:` of `{function}` is a list of two numbers, the lower limit and the upper"
                        )
                    },
                ));
            };
            if let (Some(low), Some(high)) = (low, high)
                && low >= high
            {
                return Err(refused(
                    ctx,
                    format!(
                        "the {} limit of `{function}` must be {}, but they are {} and {}",
                        if dated { "first" } else { "lower" },
                        if dated {
                            "before the last"
                        } else {
                            "below the upper"
                        },
                        told(low),
                        told(high)
                    ),
                ));
            }
            if let Some(off) = [low, high].into_iter().flatten().find(|v| !holds(*v)) {
                return Err(refused(
                    ctx,
                    format!(
                        "a limit of {} has no place on the `{name}` axis: {place}",
                        shown(off)
                    ),
                ));
            }
            (low, high)
        }
        Some(_) => return Err(internal(ctx, "limits that are not a list")),
    };
    // The breaks: a list of numbers, put in increasing order, each once.
    let breaks = match args.get(BREAKS) {
        None | Some(Value::Na) => None,
        Some(Value::List(items)) => {
            let refused = |ctx: &CallContext<'_>, why: String| {
                fail(
                    Diagnostic::error("Z0401", why)
                        .in_file(ctx.script)
                        .primary(ctx.span_of(BREAKS), "not the breaks of an axis")
                        .help(format!("write the values at which the axis has a tick and a label, as in `{sample_breaks}`")),
                )
            };
            if !dated && holds_dates(items) {
                return Err(dates_on_numbers(ctx, BREAKS, "breaks"));
            }
            let mut values: Vec<f64> = Vec::new();
            for item in items.iter() {
                match read(item) {
                    Ok(Some(v)) => values.push(v + 0.0),
                    _ => {
                        return Err(refused(
                            ctx,
                            format!(
                                "`breaks:` of `{function}` {}is a list of {things}, the values at which the axis has a tick, and each is {a_thing}",
                                if dated { "on an axis of dates " } else { "" }
                            ),
                        ));
                    }
                }
            }
            if values.is_empty() {
                return Err(refused(
                    ctx,
                    format!(
                        "`breaks:` of `{function}` is a list of {things}, but this list is empty"
                    ),
                ));
            }
            if let Some(off) = values.iter().find(|v| !holds(**v)) {
                return Err(refused(
                    ctx,
                    format!(
                        "a break at {} has no place on the `{name}` axis: {place}",
                        shown(*off)
                    ),
                ));
            }
            Some(values)
        }
        Some(_) => return Err(internal(ctx, "breaks that are not a list")),
    };
    // The labels: a text for each break, in the order the breaks are written; or, on an axis
    // of dates, one string, the pattern every break is written by.
    let mut pattern: Option<String> = None;
    let labels = match args.get(LABELS) {
        None | Some(Value::Na) => None,
        Some(Value::Str(text)) => {
            if !dated {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`labels:` of `{function}` as one string is the pattern an axis of dates is written by, and the `{name}` axis is one of numbers"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(LABELS), "a pattern for dates")
                    .help(format!("for an axis of numbers, write a label for each break, as in `{function}(breaks: [0, 1], labels: [\"no\", \"yes\"])`")),
                ));
            }
            // The pattern is tried on a day, so that a plot never holds one that cannot be
            // written.
            if let Err(why) = zeff_dates::format_with(text, 0, 0) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`labels:` of `{function}` is the pattern the dates of the axis are written by, but {why}"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(LABELS), "not a pattern for dates")
                    .help("a pattern is written as `format` writes a date: `%Y` the year, `%m` the month, `%d` the day, `%b` the name of the month, `%H:%M` the time, as in `labels: \"%b %Y\"`"),
                ));
            }
            pattern = Some(text.to_string());
            None
        }
        Some(Value::List(items)) => {
            let refused = |ctx: &CallContext<'_>, why: String| {
                fail(
                    Diagnostic::error("Z0401", why)
                        .in_file(ctx.script)
                        .primary(ctx.span_of(LABELS), "not a label for each break")
                        .help(format!("write `breaks:` and a label for each of them, in the same order, as in `{function}(breaks: [0, 1], labels: [\"no\", \"yes\"])`")),
                )
            };
            let Some(set) = &breaks else {
                return Err(refused(
                    ctx,
                    format!(
                        "`labels:` of `{function}` names the breaks one by one, so it needs `breaks:` to say where they are"
                    ),
                ));
            };
            if items.len() != set.len() {
                return Err(refused(
                    ctx,
                    format!(
                        "`{function}` has {} breaks and {} labels: each break has one label",
                        set.len(),
                        items.len()
                    ),
                ));
            }
            let mut texts: Vec<String> = Vec::new();
            for item in items.iter() {
                match item {
                    Value::Str(text) => texts.push(text.to_string()),
                    Value::Int(i) => texts.push(i.to_string()),
                    Value::Float(x) => texts.push(zeff_core::display::format_float(*x)),
                    _ => {
                        return Err(refused(
                            ctx,
                            format!(
                                "`labels:` of `{function}` is a list of text, a label for each break"
                            ),
                        ));
                    }
                }
            }
            Some(texts)
        }
        Some(_) => return Err(internal(ctx, "labels that are not a list or a string")),
    };
    // The breaks and their labels in increasing order of the breaks, each break once.
    let (breaks, labels) = match breaks {
        None => (None, None),
        Some(values) => {
            let mut order: Vec<usize> = (0..values.len()).collect();
            order.sort_by(|a, b| values[*a].total_cmp(&values[*b]));
            order.dedup_by(|a, b| values[*a] == values[*b]);
            let sorted: Vec<f64> = order.iter().map(|i| values[*i]).collect();
            let named = labels.map(|texts| order.iter().map(|i| texts[*i].clone()).collect());
            (Some(sorted), named)
        }
    };
    let mut next = (*plot).clone();
    // The scale of an axis is said once: what is said last takes the place of what was.
    next.axes.retain(|axis| axis.aesthetic != aesthetic);
    next.axes.push(PlotAxis {
        aesthetic,
        kind,
        limits,
        breaks,
        labels,
        pattern,
    });
    if let Some(fault) = axis_fault(&next) {
        return Err(misplaced(ctx, &fault, at_kind));
    }
    let before = scene(&plot).notes;
    let after = describe(&mut next);
    for note in after.notes {
        if !before.contains(&note) {
            ctx.message(Diagnostic::plain_note(note));
        }
    }
    Ok(Value::Plot(Rc::new(next)))
}
