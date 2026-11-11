//! `plot` (Language Reference, section 7.6, "Grammar"): the start of a layered plot, a frame
//! and the variables mapped to its aesthetics.
//!
//! At release 0.7.0, step 20c, the aesthetics are the positions `x` and `y` and the colours
//! `color` and `fill`. A position is a variable of the frame or an expression over it, placed on
//! a linear scale, so it must come to a Bool, an Int or a Float for each row. A colour is a
//! variable of the frame, shown as categories or on a continuous scale by its type
//! ([`crate::scene::shading`]), or a string naming one colour for every mark
//! ([`crate::colour::parse`]): a bare name is always a variable, and a string always a constant
//! (section 3.7, "Resolution by parameter type"). The four other aesthetics join at step 20d,
//! and the position scales of categories, text and dates with the marks that need them.

use zeff_columns::Column;
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Aesthetic, Frame, PlotComputed, PlotConstant, PlotMapping, PlotSetting, Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, Span};

use crate::colour::{self, NotAColour};
use crate::mapping::{Asked, computed};
use crate::scene::{is_numeric, shading, start};

const DATA: usize = 0;
const X: usize = 1;
const Y: usize = 2;
const COLOR: usize = 3;
const FILL: usize = 4;

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

/// The type of a column with its article, as a message names it: `an Int`, `a Str`.
pub(crate) fn a_type(column: &Column) -> String {
    let name = column.ty().name();
    let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {name}")
}

/// Z0401 for a string given to a colour aesthetic that names no colour.
fn not_a_colour(
    ctx: &CallContext<'_>,
    frame: &Frame,
    slot: usize,
    aesthetic: Aesthetic,
    written: &str,
    why: &NotAColour,
) -> Fail {
    let name = aesthetic.name();
    let forms = "a colour is one of the 148 names of CSS in small letters, such as \"steelblue\", or `#` and six hexadecimal digits, such as \"#4682B4\"";
    let (message, label, help) = match why {
        NotAColour::Digits => (
            format!(
                "`{name}:` of `plot` is \"{written}\", which is not a colour: after `#` come six hexadecimal digits, or three"
            ),
            "not six hexadecimal digits, or three",
            "write the red, the green and the blue as two digits each, as in \"#4682B4\"; \"#48B\" is short for \"#4488BB\"".to_string(),
        ),
        NotAColour::Name(near) => (
            format!("`{name}:` of `plot` is \"{written}\", which is not the name of a colour"),
            "not a colour",
            // The string may be the variable the script meant to colour by.
            if frame.table.column(written).is_some() {
                format!(
                    "a string sets one colour for every mark; to colour by the variable `{written}`, write it without quotation marks, `{name}: {written}`"
                )
            } else {
                match near {
                    Some(near) if near.eq_ignore_ascii_case(written) => {
                        format!("the names of colours are in small letters: write \"{near}\"")
                    }
                    Some(near) => format!("did you mean \"{near}\"?"),
                    None => forms.to_string(),
                }
            },
        ),
    };
    fail(
        Diagnostic::error("Z0401", message)
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), label)
            .help(help),
    )
}

/// `data.plot(x: Var | Expr, y: Var | Expr, color: Var | Str, fill: Var | Str)`.
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
            // An expression over the frame: computed here, once, row by row.
            Some(Value::Expr { expr, text }) => {
                let asked = Asked {
                    expr,
                    text,
                    aesthetic,
                    function: "plot",
                    span: ctx.span_of(slot),
                    own: false,
                };
                let values = computed(ctx, &frame, &asked)?;
                mappings.push(PlotMapping {
                    aesthetic,
                    variable: text.clone(),
                    computed: Some(PlotComputed {
                        expr: expr.clone(),
                        values,
                    }),
                });
                continue;
            }
            Some(_) => {
                return Err(internal(
                    ctx,
                    "`plot` was given a mapping that is not a variable or an expression",
                ));
            }
        };
        let found = column(ctx, &frame, &name, ctx.span_of(slot))?;
        if !is_numeric(found) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`plot` draws `{}` on a linear scale, so `{name}` must be a Bool, Int or Float variable, but it is {}",
                        aesthetic.name(),
                        a_type(found)
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
            computed: None,
        });
    }
    let mut constants: Vec<PlotConstant> = Vec::new();
    for (slot, aesthetic) in [(COLOR, Aesthetic::Color), (FILL, Aesthetic::Fill)] {
        match args.get(slot) {
            None | Some(Value::Na) => {}
            // A bare name: the variable whose values the colours show.
            Some(Value::Var(name)) => {
                let found = column(ctx, &frame, name, ctx.span_of(slot))?;
                if shading(found).is_none() {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`plot` shows `{}` as categories or on a continuous scale, so `{name}` must be a Bool, Int, Float, Str or Category variable, but it is {}",
                                aesthetic.name(),
                                a_type(found)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(slot), "no colour scale for this type")
                        .help(format!(
                            "a colour scale of dates is not in this release yet; `{}.describe()` lists the types of the variables",
                            frame.shown_name()
                        )),
                    ));
                }
                mappings.push(PlotMapping {
                    aesthetic,
                    variable: name.clone(),
                    computed: None,
                });
            }
            // A string: one colour for every mark.
            Some(Value::Str(written)) => match colour::parse(written) {
                Ok(hex) => constants.push(PlotConstant {
                    aesthetic,
                    value: PlotSetting::Colour {
                        hex,
                        written: written.to_string(),
                    },
                }),
                Err(why) => {
                    return Err(not_a_colour(ctx, &frame, slot, aesthetic, written, &why));
                }
            },
            Some(_) => {
                return Err(internal(
                    ctx,
                    "`plot` was given a colour that is not a variable or a string",
                ));
            }
        }
    }
    Ok(Value::Plot(std::rc::Rc::new(start(
        frame, mappings, constants,
    ))))
}
