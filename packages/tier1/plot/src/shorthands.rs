//! The shorthands (Language Reference, section 7.6, "Shorthands"; release 0.7.0, step 29):
//! the plots of a frame that most analyses begin with, each written in one call. A shorthand
//! is its grammar form and nothing else: `data.line(x, y)` is `data.plot(x: x, y: y).lines()`.
//! It calls the functions of the grammar, so the `Plot` it gives is theirs, value for value,
//! and so are its refusals, which point at the shorthand's own arguments.

use crate::grammar::{internal, plot};
use crate::{marks, scales};
use zeff_core::runtime::{Args, Body, CallContext, Fail};
use zeff_core::value::Value;
use zeff_diagnostics::Span;

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
