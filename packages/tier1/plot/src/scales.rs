//! The scales of the two axes (Language Reference, section 7.6, "Scales"): `scale_x` and
//! `scale_y` say the kind of scale of an axis of numbers, linear, log or square root, its
//! limits, its breaks and their labels. What they say is held with the plot
//! (`Plot.axes`), and the scene derives the scale from it.
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

use crate::grammar::{internal, shown};
use crate::marks::misplaced;
use crate::scene::{Placing, axis_fault, describe, placing_of, scene};

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
            "date" => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`{function}(date)` is not built yet: an axis of dates comes with the next step of this release"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(KIND), "not in this build")
                    .help("the kinds of scale built so far are `linear`, `log` and `sqrt`"),
                ));
            }
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
            .primary(ctx.callee_span, "a scale of numbers")
            .help(format!(
                "`{function}` sets the scale of an axis of numbers: its kind, its limits, its breaks and their labels"
            )),
        ));
    }
    let holds = |v: f64| match kind {
        AxisKind::Log => v > 0.0,
        AxisKind::Sqrt => v >= 0.0,
        AxisKind::Linear => true,
    };
    let place = match kind {
        AxisKind::Log => "a log scale holds the values above 0",
        AxisKind::Sqrt => "a square-root scale holds the values of 0 or more",
        AxisKind::Linear => "",
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
                        .help(format!("write the lower limit and the upper, as in `{function}(limits: [0, 100])`; `NA` leaves an end to the values drawn, as in `limits: [0, NA]`")),
                )
            };
            let [low, high] = items.as_slice() else {
                return Err(refused(
                    ctx,
                    format!(
                        "`limits:` of `{function}` is a list of two numbers, the lower limit and the upper, but this list has {}",
                        items.len()
                    ),
                ));
            };
            let (Ok(low), Ok(high)) = (number(low), number(high)) else {
                return Err(refused(
                    ctx,
                    format!(
                        "`limits:` of `{function}` is a list of two numbers, the lower limit and the upper"
                    ),
                ));
            };
            if let (Some(low), Some(high)) = (low, high)
                && low >= high
            {
                return Err(refused(
                    ctx,
                    format!(
                        "the lower limit of `{function}` must be below the upper, but they are {} and {}",
                        shown(low),
                        shown(high)
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
                        .help(format!("write the values at which the axis has a tick and a label, as in `{function}(breaks: [0, 25, 50, 75, 100])`")),
                )
            };
            let mut values: Vec<f64> = Vec::new();
            for item in items.iter() {
                match number(item) {
                    Ok(Some(v)) => values.push(v + 0.0),
                    _ => {
                        return Err(refused(
                            ctx,
                            format!(
                                "`breaks:` of `{function}` is a list of numbers, the values at which the axis has a tick"
                            ),
                        ));
                    }
                }
            }
            if values.is_empty() {
                return Err(refused(
                    ctx,
                    format!(
                        "`breaks:` of `{function}` is a list of numbers, but this list is empty"
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
    // The labels: a text for each break, in the order the breaks are written.
    let labels = match args.get(LABELS) {
        None | Some(Value::Na) => None,
        Some(Value::Str(_)) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`labels:` of `{function}` as one string is the pattern an axis of dates is written by, which is not built yet"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(LABELS), "a pattern for dates")
                .help(format!("for an axis of numbers, write a label for each break, as in `{function}(breaks: [0, 1], labels: [\"no\", \"yes\"])`")),
            ));
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
    });
    if let Some(fault) = axis_fault(&next) {
        let at = if matches!(args.get(KIND), None | Some(Value::Na)) {
            ctx.callee_span
        } else {
            ctx.span_of(KIND)
        };
        return Err(misplaced(ctx, &fault, at));
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
