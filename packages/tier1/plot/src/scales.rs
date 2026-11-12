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
use zeff_core::value::{
    Aesthetic, AxisKind, Coord, FacetScales, LegendPlace, Palette, Plot, PlotAxis, PlotFacet,
    PlotScale, PlotTheme, ThemeName, Value,
};
use zeff_diagnostics::Diagnostic;
use zeff_diagnostics::Span;

use crate::colour;
use crate::dates;
use crate::grammar::{column, internal, not_a_colour, not_a_shape, shown};
use crate::marks::{announce, misplaced};
use crate::scene::{
    Misplaced, PaletteFault, Placing, axis_fault, describe, palette_fault, placing_of, scene,
};
use crate::shape;

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
    announce(ctx, &before, &after.notes);
    Ok(Value::Plot(Rc::new(next)))
}

const S_COLOR: usize = 1;
const S_FILL: usize = 2;
const S_SIZE: usize = 3;
const S_SHAPE: usize = 4;

/// The refusal of a plot whose scale of colours cannot show its variable as the script asks.
pub(crate) fn unfit(ctx: &CallContext<'_>, fault: &PaletteFault, at: Span) -> Fail {
    let PaletteFault {
        aesthetic,
        variable,
        colours,
    } = fault;
    let (message, label) = match colours {
        None => (
            format!(
                "`okabe_ito` is the palette of categories, but `{aesthetic}` shows `{variable}` on a continuous scale"
            ),
            "a palette of categories",
        ),
        Some(count) => (
            format!(
                "`{aesthetic}` shows `{variable}` on a continuous scale, which passes through two colours or more, but the list has {count}"
            ),
            "too few colours for a ramp",
        ),
    };
    fail(
        Diagnostic::error("Z0401", message)
            .in_file(ctx.script)
            .primary(at, label)
            .help(format!("a continuous scale takes `viridis`, or the colours to pass through from its lowest value to its highest, as in `scale({aesthetic}: [\"white\", \"steelblue\"])`")),
    )
}

/// `p.scale(color: List | viridis | okabe_ito, fill: List | viridis | okabe_ito, size: List,
/// shape: List)`: what the scales that are not positions show their values as.
pub fn scale(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a scale reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let mut next = (*plot).clone();
    let mut said: Vec<(usize, Aesthetic)> = Vec::new();
    let set = |next: &mut Plot, aesthetic: Aesthetic, palette: Palette| {
        // The scale of an aesthetic is said once: what is said last takes the place of what
        // was.
        next.scales.retain(|scale| scale.aesthetic != aesthetic);
        next.scales.push(PlotScale { aesthetic, palette });
    };
    // The colours: a palette by its name, or a list of colours.
    for (slot, aesthetic) in [(S_COLOR, Aesthetic::Color), (S_FILL, Aesthetic::Fill)] {
        let name = aesthetic.name();
        let palette = match args.get(slot) {
            None | Some(Value::Na) => continue,
            Some(Value::Word(word)) => match word.as_str() {
                "viridis" => Palette::Viridis,
                "okabe_ito" => Palette::OkabeIto,
                _ => return Err(internal(ctx, "a palette that is neither of the two")),
            },
            Some(Value::List(items)) => {
                let refused = |ctx: &CallContext<'_>, why: String| {
                    fail(
                        Diagnostic::error("Z0401", why)
                            .in_file(ctx.script)
                            .primary(ctx.span_of(slot), "not a list of colours")
                            .help(format!("write the colours as strings, one for each category in their order, as in `scale({name}: [\"grey\", \"firebrick\"])`, or for a number the colours its scale passes through")),
                    )
                };
                if items.is_empty() {
                    return Err(refused(
                        ctx,
                        format!(
                            "`{name}:` of `scale` is a list of colours, but this list is empty"
                        ),
                    ));
                }
                let mut colours: Vec<(String, String)> = Vec::new();
                for item in items.iter() {
                    let Value::Str(written) = item else {
                        return Err(refused(
                            ctx,
                            format!(
                                "`{name}:` of `scale` is a list of colours, each a string, but this list holds {}",
                                item.type_word()
                            ),
                        ));
                    };
                    match colour::parse(written) {
                        Ok(hex) => colours.push((hex, written.to_string())),
                        Err(why) => {
                            return Err(not_a_colour(
                                ctx,
                                &plot.frame,
                                "scale",
                                slot,
                                aesthetic,
                                written,
                                &why,
                            ));
                        }
                    }
                }
                Palette::Colours(colours)
            }
            Some(_) => return Err(internal(ctx, "colours that are not a list or a palette")),
        };
        set(&mut next, aesthetic, palette);
        said.push((slot, aesthetic));
    }
    // The sizes: one number, the largest mark where a size shows an amount by its area from
    // zero; or two, the marks of the smallest and of the largest value drawn.
    match args.get(S_SIZE) {
        None | Some(Value::Na) => {}
        Some(Value::List(items)) => {
            let refused = |ctx: &CallContext<'_>, why: String| {
                fail(
                    Diagnostic::error("Z0401", why)
                        .in_file(ctx.script)
                        .primary(ctx.span_of(S_SIZE), "not the sizes of a scale")
                        .help("write the diameter of the largest mark, in points, as in `scale(size: [24])`, where a size shows an amount by its area; or those of the marks of the smallest and of the largest value, as in `scale(size: [2, 12])`, where it shows a difference"),
                )
            };
            let mut sizes: Vec<f64> = Vec::new();
            for item in items.iter() {
                match number(item) {
                    Ok(Some(v)) if v > 0.0 => sizes.push(v),
                    Ok(Some(v)) => {
                        return Err(refused(
                            ctx,
                            format!(
                                "a size of `scale` is the diameter of a mark, in points, above 0, but this is {}",
                                shown(v)
                            ),
                        ));
                    }
                    _ => {
                        return Err(refused(
                            ctx,
                            "`size:` of `scale` is a list of one number or of two, the diameters of marks in points".to_string(),
                        ));
                    }
                }
            }
            let palette = match sizes.as_slice() {
                [largest] => Palette::Sizes {
                    smallest: None,
                    largest: *largest,
                },
                [smallest, largest] if smallest < largest => Palette::Sizes {
                    smallest: Some(*smallest),
                    largest: *largest,
                },
                [smallest, largest] => {
                    return Err(refused(
                        ctx,
                        format!(
                            "the mark of the smallest value must be smaller than that of the largest, but the sizes are {} and {}",
                            shown(*smallest),
                            shown(*largest)
                        ),
                    ));
                }
                other => {
                    return Err(refused(
                        ctx,
                        format!(
                            "`size:` of `scale` is a list of one number or of two, but this list has {}",
                            other.len()
                        ),
                    ));
                }
            };
            set(&mut next, Aesthetic::Size, palette);
            said.push((S_SIZE, Aesthetic::Size));
        }
        Some(_) => return Err(internal(ctx, "sizes that are not a list")),
    }
    // The shapes: a name for each category, in their order.
    match args.get(S_SHAPE) {
        None | Some(Value::Na) => {}
        Some(Value::List(items)) => {
            let refused = |ctx: &CallContext<'_>, why: String| {
                fail(
                    Diagnostic::error("Z0401", why)
                        .in_file(ctx.script)
                        .primary(ctx.span_of(S_SHAPE), "not a list of shapes")
                        .help("write the names of shapes as strings, one for each category in their order, as in `scale(shape: [\"circle_open\", \"cross\"])`"),
                )
            };
            if items.is_empty() {
                return Err(refused(
                    ctx,
                    "`shape:` of `scale` is a list of shapes, but this list is empty".to_string(),
                ));
            }
            let mut shapes: Vec<&'static str> = Vec::new();
            for item in items.iter() {
                let Value::Str(written) = item else {
                    return Err(refused(
                        ctx,
                        format!(
                            "`shape:` of `scale` is a list of the names of shapes, each a string, but this list holds {}",
                            item.type_word()
                        ),
                    ));
                };
                match shape::parse(written) {
                    Ok(name) => shapes.push(name),
                    Err(near) => {
                        return Err(not_a_shape(
                            ctx,
                            &plot.frame,
                            "scale",
                            S_SHAPE,
                            written,
                            near,
                        ));
                    }
                }
            }
            set(&mut next, Aesthetic::Shape, Palette::Shapes(shapes));
            said.push((S_SHAPE, Aesthetic::Shape));
        }
        Some(_) => return Err(internal(ctx, "shapes that are not a list")),
    }
    if said.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0406", "`scale` needs `color`, `fill`, `size` or `shape`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "nothing is set")
                .help("`scale` says what a scale shows its values as: `scale(color: [\"grey\", \"firebrick\"])`, `scale(color: viridis)`, `scale(size: [2, 12])`, `scale(shape: [\"circle_open\", \"cross\"])`; the axes are set with `scale_x` and `scale_y`"),
        ));
    }
    if let Some(fault) = palette_fault(&next) {
        let at = said
            .iter()
            .find(|(_, aesthetic)| aesthetic.name() == fault.aesthetic)
            .or(said.first())
            .map_or(ctx.callee_span, |(slot, _)| ctx.span_of(*slot));
        return Err(unfit(ctx, &fault, at));
    }
    let before = scene(&plot).notes;
    let after = describe(&mut next);
    announce(ctx, &before, &after.notes);
    Ok(Value::Plot(Rc::new(next)))
}

const L_POSITION: usize = 1;

/// `p.legend(position: right | top | bottom | none | direct = right)`: where the legends of
/// the plot stand.
pub fn legend(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a legend reached its body without a plot"));
    };
    let place = match args.get(L_POSITION) {
        None | Some(Value::Na) => LegendPlace::Right,
        Some(Value::Word(word)) => match word.as_str() {
            "right" => LegendPlace::Right,
            "top" => LegendPlace::Top,
            "bottom" => LegendPlace::Bottom,
            "none" => LegendPlace::None,
            "direct" => LegendPlace::Direct,
            _ => {
                return Err(internal(
                    ctx,
                    "a place of the legend that is none of the five",
                ));
            }
        },
        Some(_) => return Err(internal(ctx, "a place of the legend that is not a word")),
    };
    let mut next = (**plot).clone();
    next.legend = place;
    describe(&mut next);
    Ok(Value::Plot(Rc::new(next)))
}

const T_TITLE: usize = 1;
const T_SUBTITLE: usize = 2;
const T_X: usize = 3;
const T_Y: usize = 4;
const T_COLOR: usize = 5;
const T_CAPTION: usize = 6;

/// `p.labels(title: Str, subtitle: Str, x: Str, y: Str, color: Str, caption: Str)`: the
/// words of the script for the plot, its two axes and its legend of colour.
pub fn labels(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "labels reached its body without a plot"));
    };
    let mut next = (**plot).clone();
    let mut said = false;
    {
        let own = &mut next.labels;
        for (slot, place) in [
            (T_TITLE, &mut own.title),
            (T_SUBTITLE, &mut own.subtitle),
            (T_X, &mut own.x),
            (T_Y, &mut own.y),
            (T_COLOR, &mut own.color),
            (T_CAPTION, &mut own.caption),
        ] {
            match args.get(slot) {
                None | Some(Value::Na) => {}
                // What is said of one of the six takes the place of what was said of it,
                // and leaves the others; an empty text takes a title away.
                Some(Value::Str(text)) => {
                    *place = Some(text.to_string());
                    said = true;
                }
                Some(_) => return Err(internal(ctx, "a label that is not text")),
            }
        }
    }
    if !said {
        return Err(fail(
            Diagnostic::error(
                "Z0406",
                "`labels` needs `title`, `subtitle`, `x`, `y`, `color` or `caption`",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "nothing is said")
            .help("`labels` gives a plot its words, as in `labels(title: \"Schooling and wages\", y: \"log wage\")`; an empty text, `x: \"\"`, takes a title away"),
        ));
    }
    describe(&mut next);
    Ok(Value::Plot(Rc::new(next)))
}

const TH_NAME: usize = 1;
const TH_BASE: usize = 2;

/// `p.theme(name: journal | minimal | slide = journal, base_size: Float = 9)`: the theme a
/// figure is drawn by.
pub fn theme(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a theme reached its body without a plot"));
    };
    let name = match args.get(TH_NAME) {
        None | Some(Value::Na) => ThemeName::Journal,
        Some(Value::Word(word)) => match word.as_str() {
            "journal" => ThemeName::Journal,
            "minimal" => ThemeName::Minimal,
            "slide" => ThemeName::Slide,
            _ => return Err(internal(ctx, "a theme that is none of the three")),
        },
        Some(_) => return Err(internal(ctx, "a theme that is not a word")),
    };
    let base_size = match args.get(TH_BASE) {
        None | Some(Value::Na) => None,
        Some(value) => {
            let size = match number(value) {
                Ok(Some(size)) => size,
                _ => return Err(internal(ctx, "a base size that is not a number")),
            };
            // The labels are eight ninths of the base size, and no text is set below 8
            // points.
            if size < crate::layout::BASE_SIZE {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`base_size:` of `theme` is the size of the titles of the axes, in points, and must be 9 or more, but this is {}",
                            shown(size)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(TH_BASE), "too small a type")
                    .help("the labels of the axes are eight ninths of the base size, and no text of a figure is set below 8 points; a figure for a slide takes `theme(slide)`, whose base size is 18"),
                ));
            }
            Some(size)
        }
    };
    let mut next = (**plot).clone();
    // What is said last of the theme takes the place of what was said.
    next.theme = PlotTheme { name, base_size };
    describe(&mut next);
    Ok(Value::Plot(Rc::new(next)))
}

/// The refusal of a rug in polar coordinates, where the call that brings the two together
/// is written.
pub(crate) fn no_rug(ctx: &CallContext<'_>, at: Span) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            "a rug has no place in polar coordinates: its ticks stand along the straight edges of a panel, and a circle has none",
        )
        .in_file(ctx.script)
        .primary(at, "a rug round a circle")
        .help("show the values as `points`, or leave out `rug` or `coord(polar)`"),
    )
}

/// The refusal of panels with axes of their own in fixed or polar coordinates, where the
/// call that brings the two together is written.
pub(crate) fn not_free(ctx: &CallContext<'_>, coord: &str, at: Span) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!("panels with axes of their own are not drawn in `{coord}` coordinates, where the scales of the plot give every panel one shape"),
        )
        .in_file(ctx.script)
        .primary(at, "free scales, where the scales shape the panel")
        .help("leave `scales:` at `fixed`, or draw the plot in `cartesian` or flipped coordinates"),
    )
}

const C_TYPE: usize = 1;
const C_RATIO: usize = 2;

/// `p.coord(type: cartesian | flip | polar | fixed = cartesian, ratio: Float)`: the
/// coordinates of a plot, how its two scales are brought to the page.
pub fn coord(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(
            ctx,
            "coordinates reached their body without a plot",
        ));
    };
    // `ratio:` belongs to `fixed`: how many units of `x` are as long as one unit of `y`.
    let ratio = match args.get(C_RATIO) {
        None | Some(Value::Na) => None,
        Some(value) => {
            let ratio = match number(value) {
                Ok(Some(ratio)) => ratio,
                _ => return Err(internal(ctx, "a ratio that is not a number")),
            };
            if !(ratio > 0.0 && ratio.is_finite()) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`ratio:` of `coord` is how many units of `x` are as long as one unit of `y`, and must be above 0, but this is {}",
                            shown(ratio)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(C_RATIO), "not a ratio")
                    .help("`coord(fixed)` gives a unit of `y` the length of a unit of `x`; `coord(fixed, ratio: 2)` makes a unit of `y` twice as long, and `ratio: 0.5` half as long"),
                ));
            }
            Some(ratio)
        }
    };
    let coord = match args.get(C_TYPE) {
        None | Some(Value::Na) => Coord::Cartesian,
        Some(Value::Word(word)) => match word.as_str() {
            "cartesian" => Coord::Cartesian,
            "flip" => Coord::Flip,
            "fixed" => Coord::Fixed(ratio.unwrap_or(1.0)),
            "polar" => Coord::Polar,
            _ => return Err(internal(ctx, "coordinates that are none of the four")),
        },
        Some(_) => return Err(internal(ctx, "coordinates that are not a word")),
    };
    if ratio.is_some() && !matches!(coord, Coord::Fixed(_)) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`ratio:` of `coord` goes with `fixed`, but the coordinates are `{}`",
                    coord.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(C_RATIO), "a ratio without `fixed`")
            .help("write `coord(fixed, ratio: 2)` for a panel in which a unit of `y` is as long as two units of `x`; the other coordinates take the shape of the page"),
        ));
    }
    let mut next = (**plot).clone();
    next.coord = coord;
    if crate::scene::rug_round_a_circle(&next) {
        return Err(no_rug(ctx, ctx.span_of(C_TYPE)));
    }
    if let Some(coord) = crate::scene::free_in_a_fixed_shape(&next) {
        return Err(not_free(ctx, coord, ctx.span_of(C_TYPE)));
    }
    describe(&mut next);
    Ok(Value::Plot(Rc::new(next)))
}

const FA_BY: usize = 1;
const FA_ROWS: usize = 2;
const FA_COLS: usize = 3;
const FA_SCALES: usize = 4;

/// `p.facet(by: Var | Vars, rows: Var, cols: Var, scales: fixed | free | free_x | free_y =
/// fixed)`: the plot parted into panels, one for each value of a variable, or for each
/// combination of the values of several.
pub fn facet(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "facets reached their body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    // The variable of the rows of a grid, and that of its columns.
    let mut grid: [Option<String>; 2] = [None, None];
    for (at, slot) in [FA_ROWS, FA_COLS].into_iter().enumerate() {
        grid[at] = match args.get(slot) {
            None | Some(Value::Na) => None,
            Some(Value::Var(name)) => {
                column(ctx, &plot.frame, name, ctx.span_of(slot))?;
                Some(name.clone())
            }
            Some(_) => {
                return Err(internal(
                    ctx,
                    "a row or column of facets that is no variable",
                ));
            }
        };
    }
    let [rows, cols] = grid;
    let shared = match args.get(FA_SCALES) {
        None | Some(Value::Na) => FacetScales::Fixed,
        Some(Value::Word(word)) => match word.as_str() {
            "fixed" => FacetScales::Fixed,
            "free" => FacetScales::Free,
            "free_x" => FacetScales::FreeX,
            "free_y" => FacetScales::FreeY,
            _ => return Err(internal(ctx, "scales of facets that are none of the four")),
        },
        Some(_) => return Err(internal(ctx, "scales of facets that are not a word")),
    };
    let gridded = rows.is_some() || cols.is_some();
    let by: Vec<String> = match args.get(FA_BY) {
        Some(Value::Var(name)) => vec![name.clone()],
        Some(Value::Vars(names)) if !names.is_empty() => names.clone(),
        None | Some(Value::Na) | Some(Value::Vars(_)) if gridded => Vec::new(),
        None | Some(Value::Na) | Some(Value::Vars(_)) => {
            return Err(fail(
                Diagnostic::error("Z0406", "`facet` needs `by`, or `rows` or `cols`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "no variable to part the plot by")
                    .help("a plot is parted into panels by a variable, one panel for each of its values, as in `facet(by: industry)`; `facet(rows: union, cols: female)` is a grid, a row of panels for each value of one variable and a column for each value of another"),
            ));
        }
        Some(_) => return Err(internal(ctx, "a `by:` of facets that names no variable")),
    };
    // Panels that follow one another, or a grid: not both.
    if gridded && !by.is_empty() {
        let slot = if rows.is_some() { FA_ROWS } else { FA_COLS };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`by:` of `facet` lays the panels one after another, and `rows:` and `cols:` lay them in a grid: a plot has one or the other",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "a grid, with `by:`")
            .help("for a grid write `facet(rows: union, cols: female)`; for a panel for each pair of values, one after another, write `facet(by: [union, female])`"),
        ));
    }
    if let (Some(down), Some(across)) = (&rows, &cols)
        && down == across
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`rows:` and `cols:` of `facet` both name `{down}`"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(FA_COLS), "the variable of the rows")
            .help(format!("a grid has one variable down the page and another across it; for one panel for each value of `{down}` write `facet(by: {down})`")),
        ));
    }
    // Each variable is one of the plot's frame, and is named once.
    for (i, name) in by.iter().enumerate() {
        column(ctx, &plot.frame, name, ctx.span_of(FA_BY))?;
        if by[..i].contains(name) {
            return Err(fail(
                Diagnostic::error("Z0401", format!("`by:` of `facet` names `{name}` twice"))
                    .in_file(ctx.script)
                    .primary(ctx.span_of(FA_BY), "one variable, twice")
                    .help("a plot is parted by each variable once; leave the second out"),
            ));
        }
    }
    let mut next = (*plot).clone();
    // What is said last of the facets takes the place of what was said.
    next.facet = Some(PlotFacet {
        by,
        rows,
        cols,
        scales: shared,
    });
    if let Some(coord) = crate::scene::free_in_a_fixed_shape(&next) {
        return Err(not_free(ctx, coord, ctx.span_of(FA_SCALES)));
    }
    let before = scene(&plot).notes;
    let after = describe(&mut next);
    announce(ctx, &before, &after.notes);
    Ok(Value::Plot(Rc::new(next)))
}
