//! The shorthands (Language Reference, section 7.6, "Shorthands"; release 0.7.0, step 29):
//! the plots of a frame that most analyses begin with, each written in one call. A shorthand
//! is its grammar form and nothing else: `data.line(x, y)` is `data.plot(x: x, y: y).lines()`.
//! It calls the functions of the grammar, so the `Plot` it gives is theirs, value for value,
//! and so are its refusals, which point at the shorthand's own arguments.

use crate::grammar::{internal, plot};
use crate::scene::{Placing, placing_of};
use crate::{marks, scales};
use zeff_core::runtime::{Args, Body, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, Mark, Plot, PlotLayer, Stat, Value};
use zeff_diagnostics::{Diagnostic, Span};

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
