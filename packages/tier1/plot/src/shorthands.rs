//! The shorthands (Language Reference, section 7.6, "Shorthands"; release 0.7.0, step 29):
//! the plots of a frame that most analyses begin with, each written in one call. A shorthand
//! is its grammar form and nothing else: `data.line(x, y)` is `data.plot(x: x, y: y).lines()`.
//! It calls the functions of the grammar, so the `Plot` it gives is theirs, value for value,
//! and so are its refusals, which point at the shorthand's own arguments.

use crate::grammar::{column, internal, plot};
use crate::scene::{Placing, number, placing_of};
use crate::{marks, scales};
use std::rc::Rc;
use zeff_columns::{Column, Table};
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, Body, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, Distribution, Frame, Mark, Plot, PlotLayer, Stat, Value};
use zeff_diagnostics::{Diagnostic, Span, count};

/// One argument of a call of the grammar made for a shorthand: its value, and where the
/// shorthand's own argument that gave it is written.
type Given = (Option<Value>, Option<Span>);

/// Nothing given for an argument.
const NONE: Given = (None, None);

/// A call of a function of the grammar on behalf of a shorthand: `given` are its arguments
/// in the order of its parameters, each with the place of the shorthand's argument it comes
/// from, so that a refusal points at what the script wrote.
fn call(ctx: &mut CallContext<'_>, body: Body, given: Vec<Given>) -> Result<Value, Fail> {
    let (values, spans): (Vec<Option<Value>>, Vec<Option<Span>>) = given.into_iter().unzip();
    let own = std::mem::replace(&mut ctx.arg_spans, spans);
    let result = body(
        ctx,
        Args {
            values,
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    );
    ctx.arg_spans = own;
    result
}

/// The argument of a shorthand at `slot`, with its place, to be handed on.
fn handed(ctx: &CallContext<'_>, args: &Args, slot: usize) -> Given {
    (
        args.get(slot).cloned(),
        ctx.arg_spans.get(slot).copied().flatten(),
    )
}

/// The plot parted into panels by `by:`, where the shorthand was given one: `facet(by:)`.
fn parted(ctx: &mut CallContext<'_>, drawn: Value, by: Given) -> Result<Value, Fail> {
    match by.0 {
        None | Some(Value::Na) => Ok(drawn),
        Some(_) => call(ctx, scales::facet, vec![(Some(drawn), None), by]),
    }
}

// ----- line and bar (step 29a) -----

const DATA: usize = 0;
const L_X: usize = 1;
const L_Y: usize = 2;
const L_COLOR: usize = 3;
const L_BY: usize = 4;

/// `data.line(x, y, color: Var, by: Var | Vars)`: `data.plot(x: x, y: y, color:).lines()`,
/// parted into panels by `by:`.
pub fn line(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if !matches!(args.get(DATA), Some(Value::Frame(_))) {
        return Err(internal(ctx, "`line` reached its body without a frame"));
    }
    let mapped = vec![
        handed(ctx, &args, DATA),
        handed(ctx, &args, L_X),
        handed(ctx, &args, L_Y),
        handed(ctx, &args, L_COLOR),
    ];
    let started = call(ctx, plot, mapped)?;
    let drawn = call(ctx, marks::lines, vec![(Some(started), None)])?;
    let by = handed(ctx, &args, L_BY);
    parted(ctx, drawn, by)
}

const B_VAR: usize = 1;
const B_COLOR: usize = 2;
const B_BY: usize = 3;

/// `data.bar(var, color: Var, by: Var | Vars)`: `data.plot(x: var, color:).bars()`, a bar
/// for each value of `var` as high as the number of its rows, parted into panels by `by:`.
pub fn bar(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if !matches!(args.get(DATA), Some(Value::Frame(_))) {
        return Err(internal(ctx, "`bar` reached its body without a frame"));
    }
    let mapped = vec![
        handed(ctx, &args, DATA),
        handed(ctx, &args, B_VAR),
        NONE,
        handed(ctx, &args, B_COLOR),
    ];
    let started = call(ctx, plot, mapped)?;
    let drawn = call(ctx, marks::bars, vec![(Some(started), None)])?;
    let by = handed(ctx, &args, B_BY);
    parted(ctx, drawn, by)
}

// ----- scatter (step 29b) -----

const S_X: usize = 1;
const S_Y: usize = 2;
const S_FIT: usize = 3;
const S_COLOR: usize = 4;
const S_BY: usize = 5;

/// The level of the band of a fitted line.
pub const BAND_LEVEL: f64 = zeff_stats::line::LEVEL;

/// How far the band of a fitted line lets through what is under it.
pub const BAND_ALPHA: f64 = 0.2;

/// A layer that draws the fitted line of the plot: its band, or the line itself.
fn fit_layer(mark: Mark, alpha: f64) -> PlotLayer {
    PlotLayer {
        mark,
        data: None,
        alpha,
        size: None,
        jitter: 0.0,
        nudge: (0.0, 0.0),
        width: None,
        stream: None,
        stat: Some(Stat::Line { level: BAND_LEVEL }),
        computed: Vec::new(),
        mappings: Vec::new(),
        constants: Vec::new(),
    }
}

/// `data.scatter(x, y, fit: none | linear = none, color: Var, by: Var | Vars)`:
/// `data.plot(x: x, y: y, color:).points()`, parted into panels by `by:`. With `fit: linear`
/// the least-squares line of `y` on `x` is drawn over the points, and its 95% confidence
/// band under them, one for each category of `color:` and each panel.
pub fn scatter(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if !matches!(args.get(DATA), Some(Value::Frame(_))) {
        return Err(internal(ctx, "`scatter` reached its body without a frame"));
    }
    let fit = match args.get(S_FIT) {
        None | Some(Value::Na) => false,
        Some(Value::Word(word)) => match word.as_str() {
            "none" => false,
            "linear" => true,
            _ => return Err(internal(ctx, "a `fit:` that is neither none nor linear")),
        },
        Some(_) => return Err(internal(ctx, "a `fit:` that is not a word")),
    };
    let mapped = vec![
        handed(ctx, &args, DATA),
        handed(ctx, &args, S_X),
        handed(ctx, &args, S_Y),
        handed(ctx, &args, S_COLOR),
    ];
    let mut drawn = call(ctx, plot, mapped)?;
    if fit {
        let Value::Plot(started) = &drawn else {
            return Err(internal(ctx, "`plot` gave no plot"));
        };
        let started: std::rc::Rc<Plot> = started.clone();
        // A line is fitted to numbers: along `x` a date counts, in days, and along `y`
        // nothing but a number does.
        for (aesthetic, slot) in [(Aesthetic::X, S_X), (Aesthetic::Y, S_Y)] {
            let (what, label) = match placing_of(&started, aesthetic) {
                Some((Placing::Categories, variable)) => (
                    format!("`{variable}` is placed as categories"),
                    "categories",
                ),
                Some((Placing::Dates, variable)) if aesthetic == Aesthetic::Y => (
                    format!("`{variable}` is placed on a scale of dates"),
                    "dates",
                ),
                _ => continue,
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`fit: linear` of `scatter` fits a line of `y` on `x`, so `{}` must be numbers, but {what}",
                        aesthetic.name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), label)
                .secondary(ctx.span_of(S_FIT), "a line is asked for here")
                .help("a least-squares line needs a number along each axis; along `x` a date will do, and is counted in days. Leave `fit:` out for the points alone"),
            ));
        }
        drawn = marks::finish(ctx, &started, fit_layer(Mark::Ribbon, BAND_ALPHA))?;
    }
    drawn = call(ctx, marks::points, vec![(Some(drawn), None)])?;
    if fit {
        let Value::Plot(pointed) = &drawn else {
            return Err(internal(ctx, "`points` gave no plot"));
        };
        let pointed: std::rc::Rc<Plot> = pointed.clone();
        drawn = marks::finish(ctx, &pointed, fit_layer(Mark::Lines, 1.0))?;
    }
    let by = handed(ctx, &args, S_BY);
    parted(ctx, drawn, by)
}

// ----- histogram, density and boxplot of a frame (step 29c2) -----

const H_VAR: usize = 1;
const H_BINS: usize = 2;
const H_BINWIDTH: usize = 3;
const H_START: usize = 4;
const H_SCALE: usize = 5;
const H_BY: usize = 6;

/// `data.histogram(var, bins:, binwidth:, start:, scale:, by:)`:
/// `data.plot(x: var).histogram(bins:, binwidth:, start:, scale:)`, parted into panels by
/// `by:`.
pub fn histogram(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if !matches!(args.get(DATA), Some(Value::Frame(_))) {
        return Err(internal(
            ctx,
            "`histogram` reached its body without a frame",
        ));
    }
    let mapped = vec![handed(ctx, &args, DATA), handed(ctx, &args, H_VAR)];
    let started = call(ctx, plot, mapped)?;
    let layer = vec![
        (Some(started), None),
        NONE,
        handed(ctx, &args, H_BINS),
        handed(ctx, &args, H_BINWIDTH),
        handed(ctx, &args, H_START),
        handed(ctx, &args, H_SCALE),
    ];
    let drawn = call(ctx, marks::histogram, layer)?;
    let by = handed(ctx, &args, H_BY);
    parted(ctx, drawn, by)
}

const D_VAR: usize = 1;
const D_COLOR: usize = 2;
const D_BANDWIDTH: usize = 3;
const D_ADJUST: usize = 4;
const D_KERNEL: usize = 5;
const D_BY: usize = 6;

/// `data.density(var, color:, bandwidth:, adjust:, kernel:, by:)`:
/// `data.plot(x: var, color:).density(bandwidth:, adjust:, kernel:)`, parted into panels by
/// `by:`.
pub fn density(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if !matches!(args.get(DATA), Some(Value::Frame(_))) {
        return Err(internal(ctx, "`density` reached its body without a frame"));
    }
    let mapped = vec![
        handed(ctx, &args, DATA),
        handed(ctx, &args, D_VAR),
        NONE,
        handed(ctx, &args, D_COLOR),
    ];
    let started = call(ctx, plot, mapped)?;
    let layer = vec![
        (Some(started), None),
        NONE,
        handed(ctx, &args, D_BANDWIDTH),
        handed(ctx, &args, D_ADJUST),
        handed(ctx, &args, D_KERNEL),
    ];
    let drawn = call(ctx, marks::density, layer)?;
    let by = handed(ctx, &args, D_BY);
    parted(ctx, drawn, by)
}

const X_VAR: usize = 1;
const X_GROUP: usize = 2;
const X_BY: usize = 3;

/// `data.boxplot(var, group:, by:)`: `data.plot(x: group, y: var).boxplot()`, a box of `var`
/// for each value of `group`, or one box without it, parted into panels by `by:`.
pub fn boxplot(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if !matches!(args.get(DATA), Some(Value::Frame(_))) {
        return Err(internal(ctx, "`boxplot` reached its body without a frame"));
    }
    let mapped = vec![
        handed(ctx, &args, DATA),
        handed(ctx, &args, X_GROUP),
        handed(ctx, &args, X_VAR),
    ];
    let started = call(ctx, plot, mapped)?;
    let drawn = call(ctx, marks::boxplot, vec![(Some(started), None)])?;
    let by = handed(ctx, &args, X_BY);
    parted(ctx, drawn, by)
}

// ----- qq_plot (step 29d) -----

const Q_VAR: usize = 1;
const Q_DISTRIBUTION: usize = 2;
const Q_BY: usize = 3;

/// The variable `qq_plot` adds to the frame it draws: for each row the quantile of the
/// distribution at the plotting position of the rank of its value.
pub const QQ_QUANTILES: &str = "theoretical";

/// The variable of the frame of the reference line that holds its two ends along `y`.
pub const QQ_LINE: &str = "line";

/// A distribution as the title of an axis names it: `normal(mean: 0, sd: 1)`.
fn named_distribution(d: &Distribution) -> String {
    match d {
        Distribution::Family { name, params } => format!(
            "{name}({})",
            params
                .iter()
                .map(|(key, value)| format!("{key}: {}", format_float(*value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => zeff_core::display::distribution_text(other),
    }
}

/// `data.qq_plot(var, distribution, by: Var | Vars)`: each value of `var` against the
/// quantile of the distribution at the plotting position of its rank, as points, with the
/// reference line through the quartiles; the standard normal where no distribution is
/// given. It is `data` with the variable `theoretical` added,
/// `.plot(x: theoretical, y: var).points()`, and `.lines(data: ends, y: line)` for the
/// reference line, whose frame holds its two ends; the pairs and the line are those of each
/// panel of `by:`.
pub fn qq_plot(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(DATA) else {
        return Err(internal(ctx, "`qq_plot` reached its body without a frame"));
    };
    let frame: Frame = frame.clone();
    let distribution = match args.get(Q_DISTRIBUTION) {
        None | Some(Value::Na) => Distribution::Family {
            name: "normal",
            params: vec![("mean", 0.0), ("sd", 1.0)],
        },
        Some(Value::Distribution(given)) => given.clone(),
        Some(_) => return Err(internal(ctx, "a `distribution` that is none")),
    };
    // The variable is read as `plot` reads what is mapped to `y`, an expression too, and
    // refused as `plot` refuses it.
    let read = vec![handed(ctx, &args, DATA), NONE, handed(ctx, &args, Q_VAR)];
    let Value::Plot(first) = call(ctx, plot, read)? else {
        return Err(internal(ctx, "`plot` gave no plot"));
    };
    if let Some((placed @ (Placing::Categories | Placing::Dates), variable)) =
        placing_of(&first, Aesthetic::Y)
    {
        let (what, label) = if placed == Placing::Categories {
            ("is placed as categories", "categories")
        } else {
            ("is placed on a scale of dates", "dates")
        };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`qq_plot` compares the values of a variable with the quantiles of a distribution, so it must be numbers, but `{variable}` {what}"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(Q_VAR), label)
            .help("give a variable of numbers, a Float, an Int or a Bool, or an expression that gives one, as in `qq_plot(log(wage))`"),
        ));
    }
    let Some(mapping) = first.mapping(Aesthetic::Y) else {
        return Err(internal(ctx, "`plot` mapped nothing to `y`"));
    };
    let values: &Column = match &mapping.computed {
        Some(computed) => &computed.values,
        None => match frame.table.column(&mapping.variable) {
            Some(found) => found,
            None => return Err(internal(ctx, "the variable of `qq_plot` is not there")),
        },
    };
    // The variables the plot is parted by: each must be there.
    let by: Vec<String> = match args.get(Q_BY) {
        Some(Value::Var(name)) => vec![name.clone()],
        Some(Value::Vars(names)) => names.clone(),
        _ => Vec::new(),
    };
    let mut parting: Vec<&Column> = Vec::new();
    for name in &by {
        parting.push(column(ctx, &frame, name, ctx.span_of(Q_BY))?);
    }
    if frame.table.column(QQ_QUANTILES).is_some() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`qq_plot` adds a variable called `{QQ_QUANTILES}` to the frame it draws, and this frame has one already"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(DATA), format!("has `{QQ_QUANTILES}`"))
            .help(format!(
                "give the variable another name first, as in `rename(expected = {QQ_QUANTILES})`"
            )),
        ));
    }
    // The rows with a finite value, group by group, in the order the groups first appear.
    let rows = values.len();
    let mut groups: Vec<(String, Vec<usize>, Vec<f64>)> = Vec::new();
    for row in 0..rows {
        let Some(value) = number(values, row).filter(|value| value.is_finite()) else {
            continue;
        };
        let key: String = parting
            .iter()
            .map(|of| format!("{:?}\u{1f}", of.get(row)))
            .collect();
        match groups.iter_mut().find(|(known, _, _)| *known == key) {
            Some((_, at, held)) => {
                at.push(row);
                held.push(value);
            }
            None => groups.push((key, vec![row], vec![value])),
        }
    }
    // For each row its quantile of the distribution; for each group the two ends of its
    // reference line, from its smallest quantile to its largest.
    let mut theoretical: Vec<Option<f64>> = vec![None; rows];
    let mut ends: Vec<(usize, f64, f64)> = Vec::new();
    let mut without = 0usize;
    for (_, at, held) in &groups {
        let found = zeff_stats::qq::pairs(held, &distribution);
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for (row, quantile) in at.iter().zip(&found.theoretical) {
            if quantile.is_finite() {
                theoretical[*row] = Some(*quantile);
                lo = lo.min(*quantile);
                hi = hi.max(*quantile);
            }
        }
        match found.line {
            Some((intercept, slope)) if lo < hi => {
                ends.push((at[0], lo, intercept + slope * lo));
                ends.push((at[0], hi, intercept + slope * hi));
            }
            _ => without += 1,
        }
    }
    // The frame of the pairs: the frame given with the quantiles beside its variables, row
    // for row, so that a point is still the row it was.
    let unbuilt =
        |ctx: &CallContext<'_>| internal(ctx, "the frame of `qq_plot` could not be built");
    let Ok(with_quantiles) = frame
        .table
        .with_column(QQ_QUANTILES, Column::floats(theoretical).0)
    else {
        return Err(unbuilt(ctx));
    };
    let pairs = Frame {
        table: Rc::new(with_quantiles),
        name: frame.name.clone(),
    };
    // The frame of the reference lines: two rows for each, with the variables of `by:`.
    let from: Vec<usize> = ends.iter().map(|end| end.0).collect();
    let mut columns: Vec<(String, Column)> = vec![
        (
            QQ_QUANTILES.to_string(),
            Column::floats(ends.iter().map(|end| Some(end.1))).0,
        ),
        (
            QQ_LINE.to_string(),
            Column::floats(ends.iter().map(|end| Some(end.2))).0,
        ),
    ];
    for (name, of) in by.iter().zip(&parting) {
        if !columns.iter().any(|(known, _)| known == name) {
            columns.push((name.clone(), of.take(&from)));
        }
    }
    let Ok(lines) = Table::new(columns) else {
        return Err(unbuilt(ctx));
    };
    let lines = Frame {
        table: Rc::new(lines),
        name: None,
    };
    // The plot: the points, the line over them, the name of the distribution along `x`.
    let mapped = vec![
        (
            Some(Value::Frame(pairs)),
            ctx.arg_spans.first().copied().flatten(),
        ),
        (Some(Value::Var(QQ_QUANTILES.to_string())), None),
        handed(ctx, &args, Q_VAR),
    ];
    let mut drawn = call(ctx, plot, mapped)?;
    drawn = call(ctx, marks::points, vec![(Some(drawn), None)])?;
    drawn = call(
        ctx,
        marks::lines,
        vec![
            (Some(drawn), None),
            (Some(Value::Frame(lines)), None),
            NONE,
            (Some(Value::Var(QQ_LINE.to_string())), None),
        ],
    )?;
    drawn = call(
        ctx,
        scales::labels,
        vec![
            (Some(drawn), None),
            NONE,
            NONE,
            (
                Some(Value::Str(format!(
                    "Quantiles of {}",
                    named_distribution(&distribution)
                ))),
                None,
            ),
        ],
    )?;
    if without > 0 {
        ctx.message(Diagnostic::plain_note(if without == 1 {
            "1 group with fewer than two values has no reference line".to_string()
        } else {
            format!(
                "{} groups with fewer than two values have no reference line",
                count(without as u64)
            )
        }));
    }
    let by = handed(ctx, &args, Q_BY);
    parted(ctx, drawn, by)
}
