//! The marks (Language Reference, section 7.6, "Marks"): each adds a layer to a plot and gives
//! the plot back, so that layers chain. At release 0.7.0, step 20d: `points`.
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
use zeff_core::value::{Aesthetic, Frame, Mark, Plot, PlotLayer, PlotMapping, PlotSetting, Value};
use zeff_diagnostics::Diagnostic;

use crate::grammar::{Slots, a_type, aesthetics, column, internal, number_option, shown};
use crate::mapping::{self, variables};
use crate::scene::{Shading, is_numeric, kind_of, scene, shading, with_layer};

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
/// layer maps to a colour of its own must be shown as the scale already shows its variable, as
/// categories or on a continuous scale. Z0401 when it is of the other kind.
fn one_scale(
    ctx: &CallContext<'_>,
    plot: &Plot,
    frame: &Frame,
    own: &[PlotMapping],
    slot: usize,
    function: &str,
) -> Result<(), Fail> {
    let Some(mapping) = own.iter().find(|m| m.aesthetic == Aesthetic::Color) else {
        return Ok(());
    };
    let Some((theirs, variable)) = kind_of(plot, Aesthetic::Color) else {
        return Ok(());
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
                "the plot shows `{variable}` {shown} for `color`, but `{name}`, which `{function}` maps to `color`, is {}{what}",
                a_type(found)
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), format!("`{name}` is of another kind"))
        .help("the layers of a plot share one scale of colour, so the variables mapped to `color` must all be categories (a Category, a Str, a Bool or a variable with value labels) or all be numbers without value labels"),
    ))
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
            values.push((*aesthetic, column));
            continue;
        }
        let found = column(ctx, &own, name, at)
            .map_err(|refusal| Box::new((*refusal).secondary(layer, draws.clone())))?;
        let whose = match &own.name {
            Some(bound) => format!("`{bound}`"),
            None => "this frame".to_string(),
        };
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
        if !is_numeric(found) {
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
            x: X,
            y: Y,
            color: COLOR,
            fill: None,
            size: SIZE,
            shape: SHAPE,
            group: None,
            label: None,
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
    one_scale(ctx, &plot, &frame, &mappings, COLOR, "points")?;
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

    let before = scene(&plot).notes;
    let (next, after) = with_layer(
        &plot,
        PlotLayer {
            mark: Mark::Points,
            data,
            alpha,
            size,
            jitter,
            stream,
            computed,
            mappings,
            constants,
        },
    );
    for note in after.notes {
        if !before.contains(&note) {
            ctx.message(Diagnostic::plain_note(note));
        }
    }
    Ok(Value::Plot(Rc::new(next)))
}
