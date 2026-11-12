//! The marks (Language Reference, section 7.6, "Marks"): each adds a layer to a plot and gives
//! the plot back, so that layers chain. At release 0.7.0: `points` (step 20d), `lines` and `step`
//! (step 23c), `area` and `ribbon` (step 23d), `hline`, `vline` and `rug`
//! (step 23e).
//!
//! A layer draws the rows of the plot's frame, or of its own (`data:`), which must then hold the
//! variables the plot maps to the aesthetics the layer reads and those its mapped expressions
//! read; an expression is computed again on such a frame, where the layer is added, and a
//! variable mapped to a colour, to the size or to the shape must be of the same kind there,
//! categories or a number, because the layers share one scale of each. A layer with a `size:` of
//! its own does not read the size the plot gives. Rows with `NA` in a mapped position, in a
//! variable shown on a continuous colour scale or in the variable the marks are sized by are not
//! drawn and are counted in a note, as everywhere else (principle 6); a second layer over the
//! same rows does not repeat the note of the first.

use std::rc::Rc;

use zeff_columns::Column;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Aesthetic, Arrangement, AxisKind, Frame, Mark, Plot, PlotConstant, PlotLayer, PlotMapping,
    PlotSetting, Sides, Stacking, Stairs, Value,
};
use zeff_diagnostics::{Diagnostic, Span};

use crate::drawing::LineType;
use crate::grammar::{Slots, a_type, aesthetics, column, internal, number_option, shown};
use crate::mapping::{self, variables};
use crate::scene::{
    AxisFault, Misplaced, Placing, Shading, axis_fault, is_numeric, kind_of, placing, placing_of,
    scene, shading, with_layer,
};

const P: usize = 0;
const DATA: usize = 1;
const X: usize = 2;
const Y: usize = 3;
const COLOR: usize = 4;
const SIZE: usize = 5;
const SHAPE: usize = 6;
const ALPHA: usize = 7;
const JITTER: usize = 8;

/// A mark placed by two positions needs something mapped to `x` and to `y`, by the plot or by
/// the layer itself (`own`); Z0401 when one is mapped by neither.
fn positions(
    ctx: &CallContext<'_>,
    plot: &Plot,
    own: &[PlotMapping],
    function: &str,
) -> Result<(), Fail> {
    let mapped = |aesthetic: Aesthetic| {
        plot.mapping(aesthetic).is_some() || own.iter().any(|m| m.aesthetic == aesthetic)
    };
    let lacking = match (mapped(Aesthetic::X), mapped(Aesthetic::Y)) {
        (true, true) => return Ok(()),
        (true, false) => "`y`",
        (false, true) => "`x`",
        (false, false) => "`x` or `y`",
    };
    // A layer that names a position of its own is told of both places a position may come from.
    let named = own
        .iter()
        .any(|m| matches!(m.aesthetic, Aesthetic::X | Aesthetic::Y));
    let (whose, help) = if named {
        (
            "neither the plot nor this layer maps a variable",
            format!(
                "name it in the layer, as in `{function}(x: educ, y: wage)`, or where the plot starts, as in `plot(x: educ, y: wage)`"
            ),
        )
    } else {
        (
            "the plot maps no variable",
            "map both where the plot starts, as in `plot(x: educ, y: wage)`".to_string(),
        )
    };
    Err(fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{function}` places each mark by its `x` and its `y`, but {whose} to {lacking}"
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "needs both `x` and `y`")
        .help(help),
    ))
}

/// The layers of a plot share one scale of each aesthetic (open decision #106): a variable a
/// layer maps to a colour of its own, `color` or `fill`, must be shown as the scale already
/// shows its variable, as categories or on a continuous scale. Z0401 when it is of the other
/// kind.
fn one_scale(
    ctx: &CallContext<'_>,
    plot: &Plot,
    frame: &Frame,
    own: &[PlotMapping],
    (aesthetic, slot): (Aesthetic, usize),
    function: &str,
) -> Result<(), Fail> {
    let Some(mapping) = own.iter().find(|m| m.aesthetic == aesthetic) else {
        return Ok(());
    };
    let Some((theirs, variable)) = kind_of(plot, aesthetic) else {
        return Ok(());
    };
    let (written, scale) = match aesthetic {
        Aesthetic::Fill => ("fill", "fill"),
        _ => ("color", "colour"),
    };
    let Some(found) = frame.table.column(&mapping.variable) else {
        return Ok(());
    };
    if shading(found) == Some(theirs) {
        return Ok(());
    }
    let name = mapping.variable.as_str();
    let labelled = found
        .metadata()
        .value_labels()
        .is_some_and(|labels| !labels.is_empty());
    let (shown, what) = match theirs {
        Shading::Categories => (
            "as categories",
            if labelled {
                " with value labels"
            } else {
                " without value labels, which is shown on a continuous scale"
            },
        ),
        Shading::Ramp => (
            "on a continuous scale",
            if labelled {
                " with value labels, which is shown as categories"
            } else {
                ", which is shown as categories"
            },
        ),
    };
    Err(fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "the plot shows `{variable}` {shown} for `{written}`, but `{name}`, which `{function}` maps to `{written}`, is {}{what}",
                a_type(found)
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), format!("`{name}` is of another kind"))
        .help(format!("the layers of a plot share one scale of {scale}, so the variables mapped to `{written}` must all be categories (a Category, a Str, a Bool or a variable with value labels) or all be numbers without value labels")),
    ))
}

/// The layers of a plot share its two axes (open decision #128): a variable a layer names for
/// `x` or `y` must be placed as the axis already places its variable, on a linear scale or as
/// categories. Z0401 when it is of the other kind. `slots` are the arguments `x:` and `y:`.
fn one_axis(
    ctx: &CallContext<'_>,
    plot: &Plot,
    frame: &Frame,
    own: &[PlotMapping],
    slots: (usize, usize),
    function: &str,
) -> Result<(), Fail> {
    for (aesthetic, slot) in [(Aesthetic::X, slots.0), (Aesthetic::Y, slots.1)] {
        let Some(mapping) = own.iter().find(|m| m.aesthetic == aesthetic) else {
            continue;
        };
        let Some((theirs, variable)) = placing_of(plot, aesthetic) else {
            continue;
        };
        let name = mapping.variable.as_str();
        // An expression is placed by what it gives.
        let (ours, what) = match (&mapping.computed, frame.table.column(name)) {
            (Some(computed), _) => match placing(&computed.values) {
                Some(Placing::Linear) => (
                    Placing::Linear,
                    "an expression, which gives numbers".to_string(),
                ),
                Some(Placing::Categories) => (
                    Placing::Categories,
                    "an expression that gives text, which is placed as categories".to_string(),
                ),
                Some(Placing::Dates) => (
                    Placing::Dates,
                    "an expression that gives dates, which are placed on a scale of dates"
                        .to_string(),
                ),
                None => continue,
            },
            (None, Some(found)) => match placing(found) {
                Some(Placing::Linear) => (
                    Placing::Linear,
                    format!("{}, which is placed on a linear scale", a_type(found)),
                ),
                Some(Placing::Categories) => (
                    Placing::Categories,
                    format!("{}, which is placed as categories", a_type(found)),
                ),
                Some(Placing::Dates) => (
                    Placing::Dates,
                    format!("{}, which is placed on a scale of dates", a_type(found)),
                ),
                None => continue,
            },
            (None, None) => continue,
        };
        if ours == theirs {
            continue;
        }
        let axis = aesthetic.name();
        let placed = match theirs {
            Placing::Linear => "on a linear scale",
            Placing::Categories => "as categories",
            Placing::Dates => "on a scale of dates",
        };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "the plot places `{variable}` {placed} along `{axis}`, but `{name}`, which `{function}` maps to `{axis}`, is {what}"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), format!("`{name}` is of another kind"))
            .help(format!(
                "the layers of a plot share one axis, so what is mapped to `{axis}` must all be categories (a Str or a Category), all be numbers or all be dates (a Date or a DateTime); `category(v)` makes categories of a number"
            )),
        ));
    }
    Ok(())
}

/// A mark that stands at numbers along an axis, an area down to zero, the edges of a band, a
/// reference line, cannot be placed on an axis of categories. Z0401 when the axis of
/// `aesthetic` is one, as the plot stands or as the layer's own mapping would make it; `why`
/// says what the mark does with its numbers. A mark that measures from zero cannot be placed
/// on a scale of dates either, which has no zero: `dates` says whether dates will do.
fn numbers_along(
    ctx: &CallContext<'_>,
    plot: &Plot,
    frame: &Frame,
    own: &[PlotMapping],
    (aesthetic, slot): (Aesthetic, Option<usize>),
    (function, why, dates): (&str, &str, bool),
) -> Result<(), Fail> {
    let axis = aesthetic.name();
    let no_dates = |ctx: &CallContext<'_>, what: String, label: String| {
        fail(
            Diagnostic::error("Z0401", what)
                .in_file(ctx.script)
                .primary(slot.map_or(ctx.callee_span, |slot| ctx.span_of(slot)), label)
                .help(format!("`points`, `lines` and `step` are placed on a scale of dates; `{function}` needs a number along `{axis}`, and may have dates along the other axis")),
        )
    };
    // The layer's own variable for the axis, where it names one, or what its expression
    // gives.
    let named = own
        .iter()
        .find(|m| m.aesthetic == aesthetic)
        .and_then(|m| match &m.computed {
            Some(computed) => Some((m, &*computed.values)),
            None => frame.table.column(&m.variable).map(|found| (m, found)),
        });
    if let Some((mapping, found)) = named {
        if placing(found) == Some(Placing::Dates) && !dates {
            return Err(no_dates(
                ctx,
                format!(
                    "`{function}` {why}, so `{axis}` must be numbers, but `{}` is {}, which is placed on a scale of dates",
                    mapping.variable,
                    a_type(found)
                ),
                "dates".to_string(),
            ));
        }
        if placing(found) != Some(Placing::Categories) {
            return Ok(());
        }
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{function}` {why}, so `{axis}` must be numbers, but `{}` is {}, which is placed as categories",
                    mapping.variable,
                    a_type(found)
                ),
            )
            .in_file(ctx.script)
            .primary(slot.map_or(ctx.callee_span, |slot| ctx.span_of(slot)), "categories")
            .help(format!("`points` and `lines` are placed on an axis of categories; `{function}` needs a number along `{axis}`")),
        ));
    }
    match placing_of(plot, aesthetic) {
        Some((Placing::Categories, variable)) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{function}` {why}, but the `{axis}` axis of this plot holds the categories of `{variable}`"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, format!("needs numbers along `{axis}`"))
            .help(format!("`points` and `lines` are placed on an axis of categories; `{function}` needs a number along `{axis}`")),
        )),
        Some((Placing::Dates, variable)) if !dates => Err(no_dates(
            ctx,
            format!(
                "`{function}` {why}, but the `{axis}` axis of this plot holds the dates of `{variable}`"
            ),
            format!("needs numbers along `{axis}`"),
        )),
        _ => Ok(()),
    }
}

/// The values a layer's own frame gives each aesthetic the plot maps from an expression.
type Computed = Vec<(Aesthetic, Rc<Column>)>;

/// The layer's own frame, if `data:` names one, with the values of the expressions the plot
/// maps computed on it: it must hold the variables the plot maps, and those its expressions
/// read, and each mapping must come to a number there.
fn own_frame(
    ctx: &mut CallContext<'_>,
    args: &Args,
    plot: &Plot,
    function: &str,
    aesthetics: &[Aesthetic],
) -> Result<(Option<Frame>, Computed), Fail> {
    let own = match args.get(DATA) {
        None | Some(Value::Na) => return Ok((None, Vec::new())),
        Some(Value::Frame(own)) => own.clone(),
        Some(_) => {
            return Err(internal(
                ctx,
                "a mark was given `data:` that is not a frame",
            ));
        }
    };
    let at = ctx.span_of(DATA);
    let layer = ctx.callee_span;
    let mut values: Computed = Vec::new();
    for aesthetic in aesthetics {
        let Some(mapping) = plot.mapping(*aesthetic) else {
            continue;
        };
        let name = mapping.variable.as_str();
        let draws = format!(
            "this layer draws `{name}`, which the plot maps to `{}`",
            aesthetic.name()
        );
        if let Some(computed) = &mapping.computed {
            for variable in variables(&computed.expr) {
                column(ctx, &own, variable, at)
                    .map_err(|refusal| Box::new((*refusal).secondary(layer, draws.clone())))?;
            }
            let asked = mapping::Asked {
                expr: &computed.expr,
                text: name,
                aesthetic: *aesthetic,
                function,
                span: at,
                own: true,
            };
            let column = mapping::computed(ctx, &own, &asked)
                .map_err(|refusal| Box::new((*refusal).secondary(layer, draws.clone())))?;
            // The layers share the axis: the expression must come to the kind of value on
            // this frame that it comes to on the plot's.
            if placing(&column) != placing(&computed.values) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "the plot maps `{name}` to `{}`, which gives {} on its own frame and {} on the frame of this layer",
                            aesthetic.name(),
                            a_type(&computed.values),
                            a_type(&column)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(at, format!("`{name}` is of another kind here"))
                    .secondary(layer, draws.clone())
                    .help("the layers of a plot share one axis, so an expression must come to numbers, to text or to dates on the frame of every layer"),
                ));
            }
            values.push((*aesthetic, column));
            continue;
        }
        let found = column(ctx, &own, name, at)
            .map_err(|refusal| Box::new((*refusal).secondary(layer, draws.clone())))?;
        let whose = match &own.name {
            Some(bound) => format!("`{bound}`"),
            None => "this frame".to_string(),
        };
        // A group is read as it stands, whatever its type: it need only be there.
        if matches!(aesthetic, Aesthetic::Group | Aesthetic::Label) {
            continue;
        }
        // A colour, a size, a shape: the variable must be shown the same way as in the plot's
        // frame, as categories or as a number, since the layers share one scale.
        if matches!(
            aesthetic,
            Aesthetic::Color | Aesthetic::Fill | Aesthetic::Size | Aesthetic::Shape
        ) {
            let theirs = plot.frame.table.column(name).and_then(shading);
            if shading(found) == theirs {
                continue;
            }
            let labelled = found
                .metadata()
                .value_labels()
                .is_some_and(|labels| !labels.is_empty());
            let (kind, what, scale, must) = match aesthetic {
                Aesthetic::Size => (
                    "by area",
                    match (shading(found), labelled) {
                        (Some(Shading::Categories), true) => {
                            " with value labels, which is categories and not an amount"
                        }
                        (Some(Shading::Categories), false) => {
                            ", which is categories and not an amount"
                        }
                        _ => ", which is not an amount",
                    },
                    "size",
                    "a number without value labels in every frame",
                ),
                Aesthetic::Shape => (
                    "as categories",
                    match shading(found) {
                        Some(Shading::Ramp) => {
                            " without value labels, which is a number and not categories"
                        }
                        _ => ", which is not categories",
                    },
                    "shape",
                    "categories in every frame (a Category, a Str, a Bool or a variable with value labels)",
                ),
                _ => (
                    match theirs {
                        Some(Shading::Categories) => "as categories",
                        Some(Shading::Ramp) => "on a continuous scale",
                        None => "on no scale",
                    },
                    match (shading(found), labelled) {
                        (Some(Shading::Categories), true) => {
                            " with value labels, which is shown as categories"
                        }
                        (Some(Shading::Categories), false) => ", which is shown as categories",
                        (Some(Shading::Ramp), _) => {
                            " without value labels, which is shown on a continuous scale"
                        }
                        (None, _) => ", which has no colour scale yet",
                    },
                    "colour",
                    "categories in every frame (a Category, a Str, a Bool or a variable with value labels) or a number without value labels in every frame",
                ),
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "the plot shows `{name}` {kind} for `{}`, but `{name}` of {whose} is {}{what}",
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(at, format!("its `{name}` is of another kind"))
                .secondary(layer, draws)
                .help(format!(
                    "the layers of a plot share one scale of {scale}, so `{name}` must be {must}"
                )),
            ));
        }
        // A position: placed as the plot's frame places it, on a linear scale or as
        // categories, since the layers share the axis.
        let theirs = plot
            .frame
            .table
            .column(name)
            .and_then(placing)
            .unwrap_or(Placing::Linear);
        if theirs == Placing::Categories && placing(found) != Some(Placing::Categories) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{function}` places `{}` as categories, so `{name}` of {whose} must be a Str or Category variable, but it is {}",
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(at, format!("its `{name}` is not categories"))
                .help(format!(
                    "the plot maps `{name}` to `{}` as categories, so the frame of every layer must hold it as categories; `category({name})` makes categories of a number",
                    aesthetic.name()
                )),
            ));
        }
        if theirs == Placing::Dates && placing(found) != Some(Placing::Dates) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{function}` places `{}` on a scale of dates, so `{name}` of {whose} must be a Date or DateTime variable, but it is {}",
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(at, format!("its `{name}` is not dates"))
                .help(format!(
                    "the plot maps `{name}` to `{}` as dates, so the frame of every layer must hold it as a Date or a DateTime; `date({name})` reads dates from text",
                    aesthetic.name()
                )),
            ));
        }
        if theirs == Placing::Linear && !is_numeric(found) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{function}` draws `{}` on a linear scale, so `{name}` of {whose} must be a Bool, Int or Float variable, but it is {}",
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(at, format!("its `{name}` is not numeric"))
                .help(format!(
                    "the plot maps `{name}` to `{}`, so the frame of every layer must hold it as a number",
                    aesthetic.name()
                )),
            ));
        }
    }
    Ok((Some(own), values))
}

/// `p.points(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, size: Var | Float,
/// shape: Var | Str, alpha: Float = 1, jitter: Float = 0)`.
pub fn points(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`points` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    // What the layer names of its own (open decision #106), read as `plot` reads it, on the
    // frame the layer draws: its own, or the plot's.
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, mut constants) = aesthetics(
        ctx,
        &args,
        &frame,
        "points",
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: Some(COLOR),
            fill: None,
            size: Some(SIZE),
            shape: Some(SHAPE),
            group: None,
            label: None,
            linetype: None,
        },
        own_data,
    )?;
    // A size that is a number is the layer's own size.
    let size = constants.iter().find_map(|c| match c.value {
        PlotSetting::Size(across) if c.aesthetic == Aesthetic::Size => Some(across),
        _ => None,
    });
    constants.retain(|c| c.aesthetic != Aesthetic::Size);
    positions(ctx, &plot, &mappings, "points")?;
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), "points")?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Color, COLOR),
        "points",
    )?;
    // What the layer reads from the plot: what it neither names nor sets of its own.
    let read: Vec<Aesthetic> = [
        Aesthetic::X,
        Aesthetic::Y,
        Aesthetic::Color,
        Aesthetic::Shape,
        Aesthetic::Size,
    ]
    .into_iter()
    .filter(|aesthetic| {
        !mappings.iter().any(|m| m.aesthetic == *aesthetic)
            && !constants.iter().any(|c| c.aesthetic == *aesthetic)
            && !(*aesthetic == Aesthetic::Size && size.is_some())
    })
    .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, "points", &read)?;

    let alpha =
        number_option(ctx, &args, ALPHA, "an `alpha:` that is not a number")?.unwrap_or(1.0);
    if !(0.0..=1.0).contains(&alpha) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`alpha:` of `points` is an opacity between 0 and 1, but this is {}",
                    shown(alpha)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(ALPHA), "outside [0, 1]")
            .help("write 1 for solid points, or a share such as 0.3 so that crowded points show through"),
        ));
    }
    let jitter =
        number_option(ctx, &args, JITTER, "a `jitter:` that is not a number")?.unwrap_or(0.0);
    if !(jitter >= 0.0 && jitter.is_finite()) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`jitter:` of `points` is a share of the smallest gap between two values, 0 or more, but this is {}",
                    shown(jitter)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(JITTER), "not a share")
            .help("write 0 for no jitter, or a share such as 0.4, which moves each point by up to 0.4 of the gap either way"),
        ));
    }
    // The stream is taken only when there is something to draw from it, so that a plot without
    // jitter neither uses the run's seed nor asks for one to be chosen.
    let stream = if jitter > 0.0 {
        Some(ctx.stream(None))
    } else {
        None
    };

    finish(
        ctx,
        &plot,
        PlotLayer {
            mark: Mark::Points,
            data,
            alpha,
            size,
            jitter,
            nudge: (0.0, 0.0),
            width: None,
            stream,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- lines (step 23c) -----

const L_COLOR: usize = 4;
const L_GROUP: usize = 5;
const L_LINETYPE: usize = 6;
const L_WIDTH: usize = 7;
const S_DIRECTION: usize = 8;

/// A line has one colour from end to end, so the colour a layer of lines draws, its own or
/// the plot's, must show categories; a number without value labels, which is shown on a ramp,
/// is refused (Z0401) with the two ways out.
fn one_colour_to_a_line(
    ctx: &CallContext<'_>,
    plot: &Plot,
    frame: &Frame,
    own: &[PlotMapping],
    set: bool,
    function: &str,
) -> Result<(), Fail> {
    let named = own.iter().find(|m| m.aesthetic == Aesthetic::Color);
    let mapping = match (named, set) {
        (Some(own), _) => own,
        (None, true) => return Ok(()),
        (None, false) => match plot.mapping(Aesthetic::Color) {
            Some(of_the_plot) => of_the_plot,
            None => return Ok(()),
        },
    };
    let Some(found) = frame.table.column(&mapping.variable) else {
        return Ok(());
    };
    if shading(found) != Some(Shading::Ramp) {
        return Ok(());
    }
    let name = mapping.variable.as_str();
    let at = if named.is_some() {
        ctx.span_of(L_COLOR)
    } else {
        ctx.callee_span
    };
    Err(fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{function}` draws each line in one colour, so `color` must show categories, but `{name}` is {} without value labels, which is shown on a continuous scale",
                a_type(found)
            ),
        )
        .in_file(ctx.script)
        .primary(at, "a colour for each row, not for each line")
        .help(format!(
            "colour the lines by a variable of categories, as in `{function}(color: union)`, or set one colour for this layer, as in `{function}(color: \"black\")`"
        )),
    ))
}

/// `p.lines(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, group: Var,
/// linetype: Var | Str, width: Float = 1)`.
pub fn lines(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    joined(ctx, args, "lines", Mark::Lines)
}

/// `p.step(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, group: Var,
/// linetype: Var | Str, width: Float = 1, direction: hv | vh | mid = hv)`.
pub fn step(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let way = match args.get(S_DIRECTION) {
        None | Some(Value::Na) => Stairs::LevelFirst,
        Some(Value::Word(word)) => match word.as_str() {
            "hv" => Stairs::LevelFirst,
            "vh" => Stairs::UprightFirst,
            "mid" => Stairs::Halfway,
            _ => return Err(internal(ctx, "a `direction:` that is none of the three")),
        },
        Some(_) => return Err(internal(ctx, "a `direction:` that is not a word")),
    };
    joined(ctx, args, "step", Mark::Step(way))
}

/// A layer of marks that are joined, a line or steps: what `lines` and `step` share.
fn joined(
    ctx: &mut CallContext<'_>,
    args: Args,
    function: &'static str,
    mark: Mark,
) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a mark reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        function,
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: Some(L_COLOR),
            fill: None,
            size: None,
            shape: None,
            group: Some(L_GROUP),
            label: None,
            linetype: Some(L_LINETYPE),
        },
        own_data,
    )?;
    positions(ctx, &plot, &mappings, function)?;
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), function)?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Color, L_COLOR),
        function,
    )?;
    // What the layer reads from the plot: what it neither names nor sets of its own.
    let read: Vec<Aesthetic> = [
        Aesthetic::X,
        Aesthetic::Y,
        Aesthetic::Color,
        Aesthetic::Group,
    ]
    .into_iter()
    .filter(|aesthetic| {
        !mappings.iter().any(|m| m.aesthetic == *aesthetic)
            && !constants.iter().any(|c| c.aesthetic == *aesthetic)
    })
    .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, function, &read)?;
    let coloured = constants.iter().any(|c| c.aesthetic == Aesthetic::Color);
    one_colour_to_a_line(ctx, &plot, &frame, &mappings, coloured, function)?;
    let width = number_option(ctx, &args, L_WIDTH, "a `width:` that is not a number")?;
    if let Some(given) = width
        && !(given > 0.0 && given.is_finite())
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`width:` of `{function}` is the width of a line in points of 1/72 inch, above 0, but this is {}",
                    shown(given)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(L_WIDTH), "not a width")
            .help("write a width such as 0.5 or 2; without `width:` a line is 1 wide"),
        ));
    }
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark,
            data,
            alpha: 1.0,
            size: None,
            jitter: 0.0,
            nudge: (0.0, 0.0),
            width,
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- area and ribbon (step 23d) -----

const A_FILL: usize = 4;
const A_GROUP: usize = 5;
const A_ALPHA: usize = 6;
const A_POSITION: usize = 7;
const R_LOW: usize = 3;
const R_HIGH: usize = 4;
const R_FILL: usize = 5;
const R_GROUP: usize = 6;
const R_ALPHA: usize = 7;

/// An area has one fill from end to end, so the fill a filled layer draws, its own or the
/// plot's, must show categories; a number without value labels, which is shown on a ramp, is
/// refused (Z0401) with the two ways out.
fn one_fill_to_an_area(
    ctx: &CallContext<'_>,
    plot: &Plot,
    frame: &Frame,
    own: &[PlotMapping],
    (set, slot): (bool, usize),
    function: &str,
) -> Result<(), Fail> {
    let named = own.iter().find(|m| m.aesthetic == Aesthetic::Fill);
    let mapping = match (named, set) {
        (Some(own), _) => own,
        (None, true) => return Ok(()),
        (None, false) => match plot.mapping(Aesthetic::Fill) {
            Some(of_the_plot) => of_the_plot,
            None => return Ok(()),
        },
    };
    let Some(found) = frame.table.column(&mapping.variable) else {
        return Ok(());
    };
    if shading(found) != Some(Shading::Ramp) {
        return Ok(());
    }
    let name = mapping.variable.as_str();
    let at = if named.is_some() {
        ctx.span_of(slot)
    } else {
        ctx.callee_span
    };
    let what = match function {
        "ribbon" => "band",
        "bars" => "bar",
        _ => "area",
    };
    Err(fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{function}` fills each {what} in one colour, so `fill` must show categories, but `{name}` is {} without value labels, which is shown on a continuous scale",
                a_type(found)
            ),
        )
        .in_file(ctx.script)
        .primary(at, format!("a colour for each row, not for each {what}"))
        .help(format!(
            "fill by a variable of categories, as in `{function}(fill: union)`, or set one fill for this layer, as in `{function}(fill: \"steelblue\")`"
        )),
    ))
}

/// `p.area(data: Frame, x: Var | Expr, y: Var | Expr, fill: Var | Str, group: Var,
/// alpha: Float = 0.6, position: overlay | stack | fill = overlay)`.
pub fn area(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let placed = match args.get(A_POSITION) {
        None | Some(Value::Na) => Stacking::Overlay,
        Some(Value::Word(word)) => match word.as_str() {
            "overlay" => Stacking::Overlay,
            "stack" => Stacking::Stack,
            "fill" => Stacking::Share,
            _ => return Err(internal(ctx, "a `position:` that is none of the three")),
        },
        Some(_) => return Err(internal(ctx, "a `position:` that is not a word")),
    };
    filled(ctx, args, "area", Mark::Area(placed))
}

/// `p.ribbon(data: Frame, x: Var | Expr, low: Var | Expr, high: Var | Expr, fill: Var | Str,
/// group: Var, alpha: Float = 0.3)`.
pub fn ribbon(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    filled(ctx, args, "ribbon", Mark::Ribbon)
}

/// A layer of filled marks, an area down to zero or a band between two variables: what `area`
/// and `ribbon` share.
fn filled(
    ctx: &mut CallContext<'_>,
    args: Args,
    function: &'static str,
    mark: Mark,
) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a mark reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let band = mark == Mark::Ribbon;
    let (fill, group, opacity, usual) = if band {
        (R_FILL, R_GROUP, R_ALPHA, 0.3)
    } else {
        (A_FILL, A_GROUP, A_ALPHA, 0.6)
    };
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        function,
        &Slots {
            x: Some(X),
            y: (!band).then_some(Y),
            low: band.then_some(R_LOW),
            high: band.then_some(R_HIGH),
            color: None,
            fill: Some(fill),
            size: None,
            shape: None,
            group: Some(group),
            label: None,
            linetype: None,
        },
        own_data,
    )?;
    let names = |aesthetic: Aesthetic| mappings.iter().any(|m| m.aesthetic == aesthetic);
    if band {
        // The two edges are the layer's own, and a band has both.
        let lacking = match (names(Aesthetic::Low), names(Aesthetic::High)) {
            (true, true) => None,
            (false, true) => Some(("`low`", "`low` is missing")),
            (true, false) => Some(("`high`", "`high` is missing")),
            (false, false) => Some(("`low` and `high`", "`low` and `high` are missing")),
        };
        if let Some((what, label)) = lacking {
            return Err(fail(
                Diagnostic::error("Z0406", format!("`ribbon` needs {what}"))
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, label)
                    .help("a band reaches from `low:` to `high:` at each row: name the two variables, as in `ribbon(low: lower, high: upper)`"),
            ));
        }
        if !names(Aesthetic::X) && plot.mapping(Aesthetic::X).is_none() {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    "`ribbon` places its band along `x`, but neither the plot nor this layer maps a variable to `x`",
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "needs `x`")
                .help("name it in the layer, as in `ribbon(x: educ, low: lower, high: upper)`, or where the plot starts, as in `plot(x: educ)`"),
            ));
        }
        numbers_along(
            ctx,
            &plot,
            &frame,
            &mappings,
            (Aesthetic::Y, None),
            (function, "places its two edges on a linear scale", false),
        )?;
    } else {
        positions(ctx, &plot, &mappings, function)?;
        numbers_along(
            ctx,
            &plot,
            &frame,
            &mappings,
            (Aesthetic::Y, Some(Y)),
            (function, "fills from its line down to zero", false),
        )?;
    }
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), function)?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Fill, fill),
        function,
    )?;
    // What the layer reads from the plot: what it neither names nor sets of its own. A band
    // does not read `y`.
    let read: Vec<Aesthetic> = [
        Aesthetic::X,
        Aesthetic::Y,
        Aesthetic::Fill,
        Aesthetic::Group,
    ]
    .into_iter()
    .filter(|aesthetic| {
        !(band && *aesthetic == Aesthetic::Y)
            && !mappings.iter().any(|m| m.aesthetic == *aesthetic)
            && !constants.iter().any(|c| c.aesthetic == *aesthetic)
    })
    .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, function, &read)?;
    let set = constants.iter().any(|c| c.aesthetic == Aesthetic::Fill);
    one_fill_to_an_area(ctx, &plot, &frame, &mappings, (set, fill), function)?;
    let alpha =
        number_option(ctx, &args, opacity, "an `alpha:` that is not a number")?.unwrap_or(usual);
    if !(0.0..=1.0).contains(&alpha) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`alpha:` of `{function}` is an opacity between 0 and 1, but this is {}",
                    shown(alpha)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(opacity), "outside [0, 1]")
            .help(format!(
                "write 1 for a fill that hides what is under it, or a share such as {} so that it shows through",
                shown(usual)
            )),
        ));
    }
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark,
            data,
            alpha,
            size: None,
            jitter: 0.0,
            nudge: (0.0, 0.0),
            width: None,
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- hline and vline (step 23e) -----

const H_AT: usize = 2;
const H_COLOR: usize = 3;
const H_STYLE: usize = 4;
const H_WIDTH: usize = 5;

/// `p.hline(data: Frame, at: Float | Var, color: Var | Str,
/// style: solid | dashed | dotted | dash_dot | long_dash | dash_dot_dot = dashed,
/// width: Float = 0.5)`.
pub fn hline(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    spanning(ctx, args, "hline", Mark::Hline)
}

/// `p.vline(...)`, with the options of `hline`.
pub fn vline(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    spanning(ctx, args, "vline", Mark::Vline)
}

/// A layer of reference lines, level or upright: what `hline` and `vline` share. The layer
/// takes nothing from the plot but its axes: its place is `at:`, a number or the values of a
/// variable, and its colour its own.
fn spanning(
    ctx: &mut CallContext<'_>,
    args: Args,
    function: &'static str,
    mark: Mark,
) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "a mark reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let (axis, along, other) = if mark == Mark::Hline {
        ("y", Aesthetic::Y, "vline")
    } else {
        ("x", Aesthetic::X, "hline")
    };
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mut mappings, mut constants) = aesthetics(
        ctx,
        &args,
        &frame,
        function,
        &Slots {
            x: None,
            y: None,
            low: None,
            high: None,
            color: Some(H_COLOR),
            fill: None,
            size: None,
            shape: None,
            group: None,
            label: None,
            linetype: None,
        },
        own_data,
    )?;
    // The place: one number, or a variable with a line at each of its values.
    match args.get(H_AT) {
        None | Some(Value::Na) => {
            return Err(fail(
                Diagnostic::error("Z0406", format!("`{function}` needs `at`"))
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`at` is missing")
                    .help(format!(
                        "a reference line stands at a number on the `{axis}` axis, as in `{function}(at: 0)`, or at each value of a variable, as in `{function}(at: wage)`"
                    )),
            ));
        }
        Some(Value::Var(name)) => {
            let found = column(ctx, &frame, name, ctx.span_of(H_AT))?;
            if !is_numeric(found) && placing(found) != Some(Placing::Dates) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`{function}` draws a line at each value of `at:` on the `{axis}` axis, so `{name}` must be a Bool, Int, Float, Date or DateTime variable, but it is {}",
                            a_type(found)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(H_AT), "not a number or a date")
                    .help(format!(
                        "a reference line stands at a number or at a date; `{}.describe()` lists the types of the variables",
                        frame.shown_name()
                    )),
                ));
            }
            mappings.push(PlotMapping {
                aesthetic: along,
                variable: name.clone(),
                computed: None,
            });
        }
        Some(at @ (Value::Int(_) | Value::Float(_) | Value::Date(_) | Value::DateTime(_))) => {
            // One place: a number, or on an axis of dates a date, held as its instant.
            let (at, dated) = match at {
                Value::Int(i) => (*i as f64, false),
                Value::Float(x) => (*x, false),
                Value::Date(days) => (f64::from(*days) * crate::dates::DAY as f64, true),
                Value::DateTime(micros) => (*micros as f64, true),
                _ => return Err(internal(ctx, "an `at:` that is a place and is not")),
            };
            if !at.is_finite() {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`at:` of `{function}` is a place on the `{axis}` axis, but this is {}",
                            shown(at)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(H_AT), "not a place on the axis")
                    .help("a reference line stands at a finite number, or at each value of a variable"),
                ));
            }
            // One line, with no row behind it to take a colour from.
            if let Some(by) = mappings.iter().find(|m| m.aesthetic == Aesthetic::Color) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`color:` of `{function}` names the variable `{}`, but `at:` is one {}: there is one line, and no row to take its colour from",
                            by.variable,
                            if dated { "date" } else { "number" }
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(H_COLOR), "a colour for each row")
                    .secondary(ctx.span_of(H_AT), "one line")
                    .help(format!(
                        "set one colour, as in `color: \"firebrick\"`, or draw a line for each row of a frame, as in `{function}(data: means, at: wage, color: union)`"
                    )),
                ));
            }
            // A place is of the kind the axis holds: a date on an axis of dates, a number
            // on one of numbers.
            match (placing_of(&plot, along), dated) {
                (Some((Placing::Dates, variable)), false) => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`at:` of `{function}` is the number {}, but the `{axis}` axis of this plot holds the dates of `{variable}`",
                                shown(at)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(H_AT), "a number on an axis of dates")
                        .help(format!(
                            "write the date, as in `{function}(at: date(\"2024-03-10\"))`, or draw a line at each date of a variable, as in `{function}(data: events, at: day)`"
                        )),
                    ));
                }
                (Some((Placing::Linear, variable)), true) => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`at:` of `{function}` is the date {}, but the `{axis}` axis of this plot holds the numbers of `{variable}`",
                                crate::dates::moment(at)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(H_AT), "a date on an axis of numbers")
                        .help(format!(
                            "a line at a date stands on an axis of dates; across the other axis the line is `{other}`"
                        )),
                    ));
                }
                _ => {}
            }
            // The line is drawn across an axis that something else gives the plot.
            if scene(&plot).scale(axis).is_none() {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`{function}` draws a line at a place on the `{axis}` axis, but nothing is mapped to `{axis}` yet"
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(
                        ctx.callee_span,
                        format!(
                            "needs {} `{axis}` axis",
                            if axis == "x" { "an" } else { "a" }
                        ),
                    )
                    .help(format!(
                        "map a variable to `{axis}` where the plot starts, or add the layer that maps it before this one; a line at a place on the other axis is `{other}`"
                    )),
                ));
            }
            constants.push(PlotConstant {
                aesthetic: along,
                value: if dated {
                    PlotSetting::Moment(at)
                } else {
                    PlotSetting::Place(at)
                },
            });
        }
        Some(_) => {
            return Err(internal(ctx, "an `at:` that is not a variable or a number"));
        }
    }
    // A reference line stands at a number: not on an axis of categories.
    numbers_along(
        ctx,
        &plot,
        &frame,
        &[],
        (along, None),
        (function, "stands at a number on its axis", true),
    )?;
    // The variable of `at:` is placed as the axis places its own: numbers with numbers, dates
    // with dates.
    one_axis(ctx, &plot, &frame, &mappings, (H_AT, H_AT), function)?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Color, H_COLOR),
        function,
    )?;
    // Nothing is read from the plot, so a frame of the layer's own needs none of its variables.
    let (data, computed) = own_frame(ctx, &args, &plot, function, &[])?;
    let style = match args.get(H_STYLE) {
        None | Some(Value::Na) => LineType::Dashed,
        Some(Value::Word(word)) => match LineType::parse(word) {
            Some(kind) => kind,
            None => return Err(internal(ctx, "a `style:` that is none of the six")),
        },
        Some(_) => return Err(internal(ctx, "a `style:` that is not a word")),
    };
    constants.push(PlotConstant {
        aesthetic: Aesthetic::Linetype,
        value: PlotSetting::Line(style.name()),
    });
    let width =
        number_option(ctx, &args, H_WIDTH, "a `width:` that is not a number")?.unwrap_or(0.5);
    if !(width > 0.0 && width.is_finite()) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`width:` of `{function}` is the width of a line in points of 1/72 inch, above 0, but this is {}",
                    shown(width)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(H_WIDTH), "not a width")
            .help("write a width such as 0.25 or 1; without `width:` a reference line is 0.5 wide"),
        ));
    }
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark,
            data,
            alpha: 1.0,
            size: None,
            jitter: 0.0,
            nudge: (0.0, 0.0),
            width: Some(width),
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- rug (step 23e2) -----

const G_COLOR: usize = 4;
const G_SIDES: usize = 5;
const G_ALPHA: usize = 6;

/// `p.rug(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str,
/// sides: x | y | both = x, alpha: Float = 1)`.
pub fn rug(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`rug` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let sides = match args.get(G_SIDES) {
        None | Some(Value::Na) => Sides::X,
        Some(Value::Word(word)) => match word.as_str() {
            "x" => Sides::X,
            "y" => Sides::Y,
            "both" => Sides::Both,
            _ => return Err(internal(ctx, "a `sides:` that is none of the three")),
        },
        Some(_) => return Err(internal(ctx, "a `sides:` that is not a word")),
    };
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        "rug",
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: Some(G_COLOR),
            fill: None,
            size: None,
            shape: None,
            group: None,
            label: None,
            linetype: None,
        },
        own_data,
    )?;
    // A rug needs the position of each of its sides, from the plot or from the layer.
    let along: &[Aesthetic] = match sides {
        Sides::X => &[Aesthetic::X],
        Sides::Y => &[Aesthetic::Y],
        Sides::Both => &[Aesthetic::X, Aesthetic::Y],
    };
    for aesthetic in along {
        let mapped = plot.mapping(*aesthetic).is_some()
            || mappings.iter().any(|m| m.aesthetic == *aesthetic);
        if mapped {
            continue;
        }
        let axis = aesthetic.name();
        let other = if *aesthetic == Aesthetic::X { "y" } else { "x" };
        // The side asked for by name is pointed at; the usual one is not written.
        let at = if matches!(args.get(G_SIDES), None | Some(Value::Na)) {
            ctx.callee_span
        } else {
            ctx.span_of(G_SIDES)
        };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`rug` draws a tick for each value of `{axis}`, but neither the plot nor this layer maps a variable to `{axis}`"
                ),
            )
            .in_file(ctx.script)
            .primary(at, format!("needs `{axis}`"))
            .help(format!(
                "map it where the plot starts, as in `plot({axis}: wage)`, or name it in the layer, as in `rug({axis}: wage)`; a rug along the other axis is `sides: {other}`"
            )),
        ));
    }
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), "rug")?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Color, G_COLOR),
        "rug",
    )?;
    // What the layer reads from the plot: the positions of its sides and the colour, where it
    // neither names nor sets them.
    let read: Vec<Aesthetic> = along
        .iter()
        .copied()
        .chain([Aesthetic::Color])
        .filter(|aesthetic| {
            !mappings.iter().any(|m| m.aesthetic == *aesthetic)
                && !constants.iter().any(|c| c.aesthetic == *aesthetic)
        })
        .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, "rug", &read)?;
    let alpha =
        number_option(ctx, &args, G_ALPHA, "an `alpha:` that is not a number")?.unwrap_or(1.0);
    if !(0.0..=1.0).contains(&alpha) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`alpha:` of `rug` is an opacity between 0 and 1, but this is {}",
                    shown(alpha)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(G_ALPHA), "outside [0, 1]")
            .help("write 1 for solid ticks, or a share such as 0.2 so that ticks on one another show how many there are"),
        ));
    }
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark: Mark::Rug(sides),
            data,
            alpha,
            size: None,
            jitter: 0.0,
            nudge: (0.0, 0.0),
            width: Some(0.5),
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- bars (step 24b) -----

const B_FILL: usize = 4;
const B_GROUP: usize = 5;
const B_POSITION: usize = 6;
const B_WIDTH: usize = 7;
const B_ALPHA: usize = 8;

/// `p.bars(data: Frame, x: Var | Expr, y: Var | Expr, fill: Var | Str, group: Var,
/// position: stack | dodge | fill = stack, width: Float = 0.9, alpha: Float = 1)`.
pub fn bars(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`bars` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let how = match args.get(B_POSITION) {
        None | Some(Value::Na) => Arrangement::Stack,
        Some(Value::Word(word)) => match word.as_str() {
            "stack" => Arrangement::Stack,
            "dodge" => Arrangement::Dodge,
            "fill" => Arrangement::Share,
            _ => return Err(internal(ctx, "a `position:` that is none of the three")),
        },
        Some(_) => return Err(internal(ctx, "a `position:` that is not a word")),
    };
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        "bars",
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: None,
            fill: Some(B_FILL),
            size: None,
            shape: None,
            group: Some(B_GROUP),
            label: None,
            linetype: None,
        },
        own_data,
    )?;
    // A bar stands at its `x`; without `y` it counts the rows there.
    if plot.mapping(Aesthetic::X).is_none() && !mappings.iter().any(|m| m.aesthetic == Aesthetic::X)
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`bars` stands a bar at each value of `x`, but neither the plot nor this layer maps a variable to `x`",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "needs `x`")
            .help("map it where the plot starts, as in `plot(x: industry)` for a bar of the count of each industry, or `plot(x: industry, y: wage)` for a bar as high as `wage`"),
        ));
    }
    numbers_along(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Y, Some(Y)),
        ("bars", "draws each bar from zero to its `y`", false),
    )?;
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), "bars")?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Fill, B_FILL),
        "bars",
    )?;
    let read: Vec<Aesthetic> = [
        Aesthetic::X,
        Aesthetic::Y,
        Aesthetic::Fill,
        Aesthetic::Group,
    ]
    .into_iter()
    .filter(|aesthetic| {
        !mappings.iter().any(|m| m.aesthetic == *aesthetic)
            && !constants.iter().any(|c| c.aesthetic == *aesthetic)
    })
    .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, "bars", &read)?;
    let set = constants.iter().any(|c| c.aesthetic == Aesthetic::Fill);
    one_fill_to_an_area(ctx, &plot, &frame, &mappings, (set, B_FILL), "bars")?;
    let width =
        number_option(ctx, &args, B_WIDTH, "a `width:` that is not a number")?.unwrap_or(0.9);
    if !(width > 0.0 && width <= 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`width:` of `bars` is the width of a bar as a share of the gap between two values of `x`, above 0 and at most 1, but this is {}",
                    shown(width)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(B_WIDTH), "not a share")
            .help("write 1 for bars that touch, or a share such as 0.5 for bars half as wide as the gap; without `width:` a bar is 0.9 of the gap"),
        ));
    }
    let alpha =
        number_option(ctx, &args, B_ALPHA, "an `alpha:` that is not a number")?.unwrap_or(1.0);
    if !(0.0..=1.0).contains(&alpha) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`alpha:` of `bars` is an opacity between 0 and 1, but this is {}",
                    shown(alpha)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(B_ALPHA), "outside [0, 1]")
            .help("write 1 for solid bars, or a share such as 0.5 so that what is under them shows through"),
        ));
    }
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark: Mark::Bars(how),
            data,
            alpha,
            size: None,
            jitter: 0.0,
            nudge: (0.0, 0.0),
            width: Some(width),
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- tile (step 24c) -----

const T_FILL: usize = 4;
const T_ALPHA: usize = 5;

/// `p.tile(data: Frame, x: Var | Expr, y: Var | Expr, fill: Var | Str, alpha: Float = 1)`.
pub fn tile(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`tile` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        "tile",
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: None,
            fill: Some(T_FILL),
            size: None,
            shape: None,
            group: None,
            label: None,
            linetype: None,
        },
        own_data,
    )?;
    positions(ctx, &plot, &mappings, "tile")?;
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), "tile")?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Fill, T_FILL),
        "tile",
    )?;
    let read: Vec<Aesthetic> = [Aesthetic::X, Aesthetic::Y, Aesthetic::Fill]
        .into_iter()
        .filter(|aesthetic| {
            !mappings.iter().any(|m| m.aesthetic == *aesthetic)
                && !constants.iter().any(|c| c.aesthetic == *aesthetic)
        })
        .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, "tile", &read)?;
    let alpha =
        number_option(ctx, &args, T_ALPHA, "an `alpha:` that is not a number")?.unwrap_or(1.0);
    if !(0.0..=1.0).contains(&alpha) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`alpha:` of `tile` is an opacity between 0 and 1, but this is {}",
                    shown(alpha)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(T_ALPHA), "outside [0, 1]")
            .help("write 1 for solid tiles, or a share such as 0.5 so that what is under them shows through"),
        ));
    }
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark: Mark::Tile,
            data,
            alpha,
            size: None,
            jitter: 0.0,
            nudge: (0.0, 0.0),
            width: None,
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

// ----- text (step 24d) -----

const X_LABEL: usize = 4;
const X_COLOR: usize = 5;
const X_SIZE: usize = 6;
const X_NUDGE: usize = 7;

/// `p.text(data: Frame, x: Var | Expr, y: Var | Expr, label: Var, color: Var | Str,
/// size: Float = 8, nudge: List)`.
pub fn text(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`text` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let frame: Frame = match args.get(DATA) {
        Some(Value::Frame(own)) => own.clone(),
        _ => plot.frame.clone(),
    };
    let own_data = matches!(args.get(DATA), Some(Value::Frame(_)));
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        "text",
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: Some(X_COLOR),
            fill: None,
            size: None,
            shape: None,
            group: None,
            label: Some(X_LABEL),
            linetype: None,
        },
        own_data,
    )?;
    positions(ctx, &plot, &mappings, "text")?;
    // What is written: the plot's `label`, or the layer's own.
    if plot.mapping(Aesthetic::Label).is_none()
        && !mappings.iter().any(|m| m.aesthetic == Aesthetic::Label)
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`text` writes the `label` of each row at its place, but neither the plot nor this layer maps a variable to `label`",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "needs `label`")
            .help("name the variable whose values are written, as in `text(label: name)`, or where the plot starts, as in `plot(x: educ, y: wage, label: name)`"),
        ));
    }
    one_axis(ctx, &plot, &frame, &mappings, (X, Y), "text")?;
    one_scale(
        ctx,
        &plot,
        &frame,
        &mappings,
        (Aesthetic::Color, X_COLOR),
        "text",
    )?;
    let read: Vec<Aesthetic> = [
        Aesthetic::X,
        Aesthetic::Y,
        Aesthetic::Color,
        Aesthetic::Label,
    ]
    .into_iter()
    .filter(|aesthetic| {
        !mappings.iter().any(|m| m.aesthetic == *aesthetic)
            && !constants.iter().any(|c| c.aesthetic == *aesthetic)
    })
    .collect();
    let (data, computed) = own_frame(ctx, &args, &plot, "text", &read)?;
    let size = number_option(ctx, &args, X_SIZE, "a `size:` that is not a number")?;
    if let Some(given) = size
        && !(given > 0.0 && given.is_finite())
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`size:` of `text` is the size of the type in points of 1/72 inch, above 0, but this is {}",
                    shown(given)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(X_SIZE), "not a size")
            .help("write a size such as 8 or 10; without `size:` the text is 8 points, as the labels of the axes are"),
        ));
    }
    // How far the labels are moved: to the right and up, in points.
    let nudge = match args.get(X_NUDGE) {
        None | Some(Value::Na) => (0.0, 0.0),
        Some(Value::List(items)) => {
            let number = |value: &Value| match value {
                Value::Int(i) => Some(*i as f64),
                Value::Float(x) if x.is_finite() => Some(*x),
                _ => None,
            };
            match items.as_slice() {
                [right, up] => match (number(right), number(up)) {
                    (Some(right), Some(up)) => (right, up),
                    _ => return Err(not_a_nudge(ctx)),
                },
                _ => return Err(not_a_nudge(ctx)),
            }
        }
        Some(_) => return Err(not_a_nudge(ctx)),
    };
    finish(
        ctx,
        &plot,
        PlotLayer {
            mark: Mark::Text,
            data,
            alpha: 1.0,
            size,
            jitter: 0.0,
            nudge,
            width: None,
            stream: None,
            computed,
            mappings,
            constants,
        },
    )
}

/// The refusal of a `nudge:` that is not two finite numbers.
fn not_a_nudge(ctx: &CallContext<'_>) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            "`nudge:` of `text` is a list of two numbers: how far the labels are moved to the right and up, in points of 1/72 inch",
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(X_NUDGE), "not two numbers")
        .help("write `nudge: [0, 6]` to write each label 6 points above its place, clear of a point drawn there; a number below zero moves it left or down"),
    )
}

// ----- what every mark does last -----

/// Add a layer to a plot and give the plot back: what every mark does last. The plot must
/// still be one that its scales can draw: a value at or below zero has no place on a log
/// axis, nor a value below zero on a square-root axis, and such a plot is refused with the
/// rows named (Z0401; section 7.6, "Scales"). The notes of the rows the new layer does not
/// draw are said once, where the layer is added.
fn finish(ctx: &mut CallContext<'_>, plot: &Plot, layer: PlotLayer) -> Result<Value, Fail> {
    let before = scene(plot).notes;
    let (next, after) = with_layer(plot, layer);
    if let Some(fault) = axis_fault(&next) {
        return Err(misplaced(ctx, &fault, ctx.callee_span));
    }
    for note in after.notes {
        if !before.contains(&note) {
            ctx.message(Diagnostic::plain_note(note));
        }
    }
    Ok(Value::Plot(Rc::new(next)))
}

/// The refusal of a plot that holds something with no place on the scale of an axis.
pub(crate) fn misplaced(ctx: &CallContext<'_>, fault: &AxisFault, at: Span) -> Fail {
    let axis = fault.axis;
    let (scale, limit, keep) = match fault.kind {
        AxisKind::Log => ("a log scale", "at or below 0", "above 0"),
        AxisKind::Date => ("a scale of dates", "", ""),
        _ => ("a square-root scale", "below 0", "of 0 or more"),
    };
    // What the scale asked for is set on: an axis of numbers, or one of dates.
    let (holds, set_on) = if fault.kind == AxisKind::Date {
        (
            "dates",
            "a scale of dates is set on an axis that a Date or a DateTime variable is mapped to",
        )
    } else {
        (
            "numbers",
            "a log or a square-root scale is set on an axis of numbers",
        )
    };
    let other = match fault.kind {
        AxisKind::Log => "; a square-root scale, `sqrt`, holds 0",
        _ => "",
    };
    let diagnostic = match &fault.what {
        Misplaced::Rows {
            variable,
            frame,
            rows,
            count,
        } => {
            let listed: Vec<String> = rows.iter().map(|row| count_of(*row)).collect();
            let named = match (listed.split_last(), count - rows.len()) {
                (Some((only, [])), 0) => format!("row {only}"),
                (Some((last, rest)), 0) => format!("rows {} and {last}", rest.join(", ")),
                (_, more) => format!("rows {} and {} more", listed.join(", "), count_of(more)),
            };
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{variable}` has {} {limit}, which {} no place on {scale}: {named} of {frame}",
                    if *count == 1 {
                        "1 value".to_string()
                    } else {
                        format!("{} values", count_of(*count))
                    },
                    if *count == 1 { "has" } else { "have" }
                ),
            )
            .primary(at, format!("the `{axis}` axis is {scale}"))
            .help(format!(
                "keep the rows with a value {keep}, as in `where({variable} {} 0)`{other}",
                if fault.kind == AxisKind::Log { ">" } else { ">=" }
            ))
        }
        Misplaced::Place(place) => Diagnostic::error(
            "Z0401",
            format!(
                "a reference line at {} has no place on {scale}, which holds the values {keep}",
                shown(*place)
            ),
        )
        .primary(at, format!("the `{axis}` axis is {scale}"))
        .help(format!("place the line at a value {keep}")),
        Misplaced::Zero(mark) => Diagnostic::error(
            "Z0401",
            format!(
                "`{mark}` reaches down to zero, and zero has no place on a log scale"
            ),
        )
        .primary(at, format!("the `{axis}` axis is a log scale"))
        .help("a square-root scale, `sqrt`, holds zero; on a log scale `points` or `lines` show the values"),
        Misplaced::Categories(variable) => Diagnostic::error(
            "Z0401",
            format!(
                "the `{axis}` axis holds the categories of `{variable}`, which have their places and no other scale"
            ),
        )
        .primary(at, format!("{scale} is for {holds}"))
        .help(set_on),
        Misplaced::Dates(variable) if fault.kind == AxisKind::Linear => Diagnostic::error(
            "Z0401",
            format!(
                "the `{axis}` axis holds the dates of `{variable}`, but the limits or the breaks set for it are numbers"
            ),
        )
        .primary(at, "dates on an axis set in numbers")
        .help(format!("set them as dates, as in `scale_{axis}(date, limits: [date(\"2024-01-01\"), NA])`")),
        Misplaced::Dates(variable) => Diagnostic::error(
            "Z0401",
            format!(
                "the `{axis}` axis holds the dates of `{variable}`, which are placed in proportion to the time and on no other scale"
            ),
        )
        .primary(at, format!("{scale} is for numbers"))
        .help("a log or a square-root scale is set on an axis of numbers"),
        Misplaced::Numbers(variable) => Diagnostic::error(
            "Z0401",
            format!(
                "the `{axis}` axis holds the numbers of `{variable}`, which a scale of dates has no place for"
            ),
        )
        .primary(at, "a scale of dates is for dates")
        .help("a scale of dates holds a Date or a DateTime variable; `date(v)` reads dates from text"),
    };
    fail(diagnostic.in_file(ctx.script))
}

/// A count as the messages write it.
fn count_of(n: usize) -> String {
    zeff_diagnostics::count(n as u64)
}
