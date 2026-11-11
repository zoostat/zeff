//! The marks (Language Reference, section 7.6, "Marks"): each adds a layer to a plot and gives
//! the plot back, so that layers chain. At release 0.7.0, step 20c: `points`.
//!
//! A layer draws the rows of the plot's frame, or of its own (`data:`), which must then hold the
//! variables the plot maps to the aesthetics its mark reads and those its mapped expressions
//! read; an expression is computed again on such a frame, where the layer is added, and a
//! variable mapped to a colour must be of the same kind there, categories or a number, because
//! the layers share one colour scale. Rows with `NA` in a mapped position, or in a variable shown
//! on a continuous colour scale, are not drawn and are counted in a note, as everywhere else
//! (principle 6); a second layer over the same rows does not repeat the note of the first.

use std::rc::Rc;

use zeff_columns::Column;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, Frame, Mark, Plot, PlotLayer, Value};
use zeff_diagnostics::Diagnostic;

use crate::grammar::{a_type, column, internal, number_option, shown};
use crate::mapping::{self, variables};
use crate::scene::{Shading, is_numeric, scene, shading, with_layer};

const P: usize = 0;
const DATA: usize = 1;
const ALPHA: usize = 2;
const SIZE: usize = 3;
const JITTER: usize = 4;

/// A mark placed by two positions needs something mapped to `x` and to `y`; Z0401 when one is
/// not mapped.
fn positions(ctx: &CallContext<'_>, plot: &Plot, function: &str) -> Result<(), Fail> {
    let lacking = match (plot.mapped(Aesthetic::X), plot.mapped(Aesthetic::Y)) {
        (Some(_), Some(_)) => return Ok(()),
        (Some(_), None) => "`y`",
        (None, Some(_)) => "`x`",
        (None, None) => "`x` or `y`",
    };
    Err(fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`{function}` places each mark by its `x` and its `y`, but the plot maps no variable to {lacking}"
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "needs both `x` and `y`")
        .help("map both where the plot starts, as in `plot(x: educ, y: wage)`"),
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
        // A colour: the variable must be shown the same way as in the plot's frame, as
        // categories or on a continuous scale, since the layers share one scale.
        if matches!(aesthetic, Aesthetic::Color | Aesthetic::Fill) {
            let theirs = plot.frame.table.column(name).and_then(shading);
            if shading(found) == theirs {
                continue;
            }
            let kind = |shading: Option<Shading>| match shading {
                Some(Shading::Categories) => "as categories",
                Some(Shading::Ramp) => "on a continuous scale",
                None => "on no scale",
            };
            let labelled = found
                .metadata()
                .value_labels()
                .is_some_and(|labels| !labels.is_empty());
            let what = match (shading(found), labelled) {
                (Some(Shading::Categories), true) => {
                    " with value labels, which is shown as categories"
                }
                (Some(Shading::Categories), false) => ", which is shown as categories",
                (Some(Shading::Ramp), _) => {
                    " without value labels, which is shown on a continuous scale"
                }
                (None, _) => ", which has no colour scale yet",
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "the plot shows `{name}` {} for `{}`, but `{name}` of {whose} is {}{what}",
                        kind(theirs),
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(at, format!("its `{name}` is of another kind"))
                .secondary(layer, draws)
                .help(format!(
                    "the layers of a plot share one scale of colour, so `{name}` must be categories in every frame (a Category, a Str, a Bool or a variable with value labels) or a number without value labels in every frame"
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

/// `p.points(data: Frame, alpha: Float = 1, size: Float, jitter: Float = 0)`.
pub fn points(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`points` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    positions(ctx, &plot, "points")?;
    let (data, computed) = own_frame(
        ctx,
        &args,
        &plot,
        "points",
        &[Aesthetic::X, Aesthetic::Y, Aesthetic::Color],
    )?;

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
    let size = number_option(ctx, &args, SIZE, "a `size:` that is not a number")?;
    if let Some(given) = size
        && !(given > 0.0 && given.is_finite())
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`size:` of `points` is the diameter of a point in points of 1/72 inch, above 0, but this is {}",
                    shown(given)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(SIZE), "not a size")
            .help("write a size such as 3 or 6; without `size:` a point is 4.5 across"),
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
        },
    );
    for note in after.notes {
        if !before.contains(&note) {
            ctx.message(Diagnostic::plain_note(note));
        }
    }
    Ok(Value::Plot(Rc::new(next)))
}
