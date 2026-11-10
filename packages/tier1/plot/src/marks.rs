//! The marks (Language Reference, section 7.6, "Marks"): each adds a layer to a plot and gives
//! the plot back, so that layers chain. At release 0.7.0, step 20: `points`.
//!
//! A layer draws the rows of the plot's frame, or of its own (`data:`), which must then hold the
//! variables the plot maps. Rows with `NA` in a mapped variable are not drawn and are counted in
//! a note, as everywhere else (principle 6); a second layer over the same rows does not repeat
//! the note of the first.

use std::rc::Rc;

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, Frame, Mark, Plot, PlotLayer, Value};
use zeff_diagnostics::Diagnostic;

use crate::grammar::{column, internal, number_option, shown};
use crate::scene::{is_numeric, scene, with_layer};

const P: usize = 0;
const DATA: usize = 1;
const ALPHA: usize = 2;
const SIZE: usize = 3;
const JITTER: usize = 4;

/// The variables the plot maps to `x` and `y`, both of which a mark placed by two positions
/// needs; Z0401 when one is not mapped.
fn positions<'a>(
    ctx: &CallContext<'_>,
    plot: &'a Plot,
    function: &str,
) -> Result<(&'a str, &'a str), Fail> {
    let lacking = match (plot.mapped(Aesthetic::X), plot.mapped(Aesthetic::Y)) {
        (Some(x), Some(y)) => return Ok((x, y)),
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

/// The layer's own frame, if `data:` names one: it must hold the variables the plot maps, as
/// numbers.
fn own_frame(
    ctx: &CallContext<'_>,
    args: &Args,
    function: &str,
    mapped: &[(Aesthetic, &str)],
) -> Result<Option<Frame>, Fail> {
    let own = match args.get(DATA) {
        None | Some(Value::Na) => return Ok(None),
        Some(Value::Frame(own)) => own.clone(),
        Some(_) => {
            return Err(internal(
                ctx,
                "a mark was given `data:` that is not a frame",
            ));
        }
    };
    for (aesthetic, name) in mapped {
        let found = column(ctx, &own, name, ctx.span_of(DATA)).map_err(|refusal| {
            Box::new((*refusal).secondary(
                ctx.callee_span,
                format!(
                    "this layer draws `{name}`, which the plot maps to `{}`",
                    aesthetic.name()
                ),
            ))
        })?;
        if !is_numeric(found) {
            let whose = match &own.name {
                Some(bound) => format!("`{bound}`"),
                None => "this frame".to_string(),
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{function}` draws `{}` on a linear scale, so `{name}` of {whose} must be a Bool, Int or Float variable, but it is a {}",
                        aesthetic.name(),
                        found.ty().name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(DATA), format!("its `{name}` is not numeric"))
                .help(format!(
                    "the plot maps `{name}` to `{}`, so the frame of every layer must hold it as a number",
                    aesthetic.name()
                )),
            ));
        }
    }
    Ok(Some(own))
}

/// `p.points(data: Frame, alpha: Float = 1, size: Float, jitter: Float = 0)`.
pub fn points(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Plot(plot)) = args.get(P) else {
        return Err(internal(ctx, "`points` reached its body without a plot"));
    };
    let plot: Rc<Plot> = plot.clone();
    let (x, y) = positions(ctx, &plot, "points")?;
    let data = own_frame(
        ctx,
        &args,
        "points",
        &[(Aesthetic::X, x), (Aesthetic::Y, y)],
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
        },
    );
    for note in after.notes {
        if !before.contains(&note) {
            ctx.message(Diagnostic::plain_note(note));
        }
    }
    Ok(Value::Plot(Rc::new(next)))
}
