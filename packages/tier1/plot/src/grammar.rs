//! `plot` (Language Reference, section 7.6, "Grammar"): the start of a layered plot, a frame
//! and the variables mapped to its aesthetics.
//!
//! At release 0.7.0, step 20, the aesthetics are `x` and `y`, each a variable of the frame placed
//! on a linear scale, so each must be a Bool, an Int or a Float. Expressions and the six other
//! aesthetics join at steps 20b and 20c, and the scales of categories, text and dates with the
//! marks that need them.

use zeff_columns::Column;
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, Frame, PlotMapping, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, Span};

use crate::scene::{is_numeric, start};

const DATA: usize = 0;
const X: usize = 1;
const Y: usize = 2;

/// Z9001: a body reached in a state the checker rules out.
pub(crate) fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// The variable `name` of `frame`, or Z0101 or Z0102 at `span` when the frame has none.
pub(crate) fn column<'a>(
    ctx: &CallContext<'_>,
    frame: &'a Frame,
    name: &str,
    span: Span,
) -> Result<&'a Column, Fail> {
    match frame.table.column(name) {
        Some(column) => Ok(column),
        None => {
            let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
            Err(fail(no_variable(
                ctx.script,
                span,
                &frame.shown_name(),
                name,
                &columns,
            )))
        }
    }
}

/// A number option as a Float: `None` when it was not given.
pub(crate) fn number_option(
    ctx: &CallContext<'_>,
    args: &Args,
    slot: usize,
    what: &str,
) -> Result<Option<f64>, Fail> {
    match args.get(slot) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Float(x)) => Ok(Some(*x)),
        Some(Value::Int(i)) => Ok(Some(*i as f64)),
        Some(_) => Err(internal(ctx, what)),
    }
}

/// A number as a message shows it.
pub(crate) fn shown(x: f64) -> String {
    format_float(x)
}

/// `data.plot(x: Var, y: Var)`.
pub fn plot(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(DATA) else {
        return Err(internal(ctx, "`plot` reached its body without a frame"));
    };
    let frame: Frame = frame.clone();
    let mut mappings: Vec<PlotMapping> = Vec::new();
    for (slot, aesthetic) in [(X, Aesthetic::X), (Y, Aesthetic::Y)] {
        let name = match args.get(slot) {
            None | Some(Value::Na) => continue,
            Some(Value::Var(name)) => name.clone(),
            Some(_) => {
                return Err(internal(
                    ctx,
                    "`plot` was given a mapping that is not a variable",
                ));
            }
        };
        let found = column(ctx, &frame, &name, ctx.span_of(slot))?;
        if !is_numeric(found) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`plot` draws `{}` on a linear scale, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                        aesthetic.name(),
                        found.ty().name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), "not a numeric variable")
                .help(format!(
                    "the scales of categories, text and dates are not in this release yet; `{}.describe()` lists the types of the variables",
                    frame.shown_name()
                )),
            ));
        }
        mappings.push(PlotMapping {
            aesthetic,
            variable: name,
        });
    }
    Ok(Value::Plot(std::rc::Rc::new(start(frame, mappings))))
}
