//! `plot` (Language Reference, section 7.6, "Grammar"): the start of a layered plot, a frame
//! and the variables mapped to its aesthetics.
//!
//! A position, `x` or `y`, is a variable of the frame or an expression over it, placed on a
//! linear scale, so it must come to a Bool, an Int or a Float for each row. A colour, `color` or
//! `fill`, is a variable of the frame, shown as categories or on a continuous scale by its type
//! ([`crate::scene::shading`]), or a string naming one colour for every mark
//! ([`crate::colour::parse`]). A `size` is a variable that is an amount, an Int or a Float
//! without value labels, shown by the area of the mark, or a number, the diameter of every
//! mark. A `shape` is a variable shown as categories, each with a shape of its own, or a string
//! naming one shape for every mark ([`crate::shape::parse`]). A bare name is always a variable,
//! and a string or a number always a constant (section 3.7, "Resolution by parameter type").
//! `group` and `label` are variables of the frame of any type, held for the marks that read
//! them. The position scales of categories, text and dates join with the marks that need them.

use zeff_columns::Column;
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Aesthetic, Frame, PlotComputed, PlotConstant, PlotMapping, PlotSetting, Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, Span};

use crate::colour::{self, NotAColour};
use crate::drawing::LineType;
use crate::mapping::{Asked, computed};
use crate::scene::{Shading, is_numeric, on_categories, shading, start};
use crate::shape;

const DATA: usize = 0;
const X: usize = 1;
const Y: usize = 2;
const COLOR: usize = 3;
const FILL: usize = 4;
const SIZE: usize = 5;
const SHAPE: usize = 6;
const GROUP: usize = 7;
const LABEL: usize = 8;

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
    function: &str,
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
                "`{name}:` of `{function}` is \"{written}\", which is not a colour: after `#` come six hexadecimal digits, or three"
            ),
            "not six hexadecimal digits, or three",
            "write the red, the green and the blue as two digits each, as in \"#4682B4\"; \"#48B\" is short for \"#4488BB\"".to_string(),
        ),
        NotAColour::Name(near) => (
            format!("`{name}:` of `{function}` is \"{written}\", which is not the name of a colour"),
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

/// Whether a variable carries value labels, which make a number a set of names.
fn labelled(column: &Column) -> bool {
    column
        .metadata()
        .value_labels()
        .is_some_and(|labels| !labels.is_empty())
}

/// Z0401 for a string given to `shape:` that names no shape.
fn not_a_shape(
    ctx: &CallContext<'_>,
    frame: &Frame,
    function: &str,
    slot: usize,
    written: &str,
    near: Option<&'static str>,
) -> Fail {
    // The string may be the variable the script meant to give shapes by.
    let help = if frame.table.column(written).is_some() {
        format!(
            "a string sets one shape for every mark; to give each category of `{written}` a shape, write it without quotation marks, `shape: {written}`"
        )
    } else {
        match near {
            Some(near) if near.eq_ignore_ascii_case(written) => {
                format!("the names of shapes are in small letters: write \"{near}\"")
            }
            Some(near) => format!("did you mean \"{near}\"?"),
            None => {
                let (last, others) = shape::NAMED.split_last().unwrap_or((&"", &[]));
                format!("a shape is one of {} and {last}", others.join(", "))
            }
        }
    };
    fail(
        Diagnostic::error(
            "Z0401",
            format!("`shape:` of `{function}` is \"{written}\", which is not the name of a shape"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "not a shape")
        .help(help),
    )
}

/// Where the aesthetics stand among the arguments of a call: the two positions, a colour, a
/// size and a shape, which every mark of points takes, and those only some calls take.
pub(crate) struct Slots {
    pub x: Option<usize>,
    pub y: Option<usize>,
    /// The two edges of a band, for the mark that has them.
    pub low: Option<usize>,
    pub high: Option<usize>,
    pub color: Option<usize>,
    pub fill: Option<usize>,
    pub size: Option<usize>,
    pub shape: Option<usize>,
    pub group: Option<usize>,
    pub label: Option<usize>,
    pub linetype: Option<usize>,
}

/// What a call maps and what it sets to one value, read from its arguments as `plot` reads
/// them (open decision #106): a bare name is a variable of `frame`, an expression given to a
/// position is computed on it, and a string or a number sets a colour, a shape or a size for
/// every mark. `function` is the call, for the messages; `own` says that `frame` is a layer's
/// own.
pub(crate) fn aesthetics(
    ctx: &mut CallContext<'_>,
    args: &Args,
    frame: &Frame,
    function: &'static str,
    slots: &Slots,
    own: bool,
) -> Result<(Vec<PlotMapping>, Vec<PlotConstant>), Fail> {
    let mut mappings: Vec<PlotMapping> = Vec::new();
    for (slot, aesthetic) in [
        (slots.x, Aesthetic::X),
        (slots.y, Aesthetic::Y),
        (slots.low, Aesthetic::Low),
        (slots.high, Aesthetic::High),
    ] {
        let Some(slot) = slot else {
            continue;
        };
        let name = match args.get(slot) {
            None | Some(Value::Na) => continue,
            Some(Value::Var(name)) => name.clone(),
            // An expression over the frame: computed here, once, row by row.
            Some(Value::Expr { expr, text }) => {
                let asked = Asked {
                    expr,
                    text,
                    aesthetic,
                    function,
                    span: ctx.span_of(slot),
                    own,
                };
                let values = computed(ctx, frame, &asked)?;
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
                    "a call was given a mapping that is not a variable or an expression",
                ));
            }
        };
        let found = column(ctx, frame, &name, ctx.span_of(slot))?;
        // A position is a number, on a linear scale, or a Str or a Category, on an axis of
        // categories (open decision #128); the edges of a band are numbers.
        let position = matches!(aesthetic, Aesthetic::X | Aesthetic::Y);
        if position && !is_numeric(found) && !on_categories(found) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{function}` places `{}` on a linear scale or on an axis of categories, so `{name}` must be a Bool, Int, Float, Str or Category variable, but it is {}",
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), "no axis for this type")
                .help(format!(
                    "an axis of dates is not in this release yet; `{}.describe()` lists the types of the variables",
                    frame.shown_name()
                )),
            ));
        }
        if !position && !is_numeric(found) {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{function}` draws `{}` on a linear scale, so `{name}` must be a Bool, Int or Float variable, but it is {}",
                        aesthetic.name(),
                        a_type(found)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), "not a numeric variable")
                .help(format!(
                    "the edges of a band are numbers; `{}.describe()` lists the types of the variables",
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
    for (slot, aesthetic) in [
        (slots.color, Aesthetic::Color),
        (slots.fill, Aesthetic::Fill),
    ] {
        let Some(slot) = slot else {
            continue;
        };
        match args.get(slot) {
            None | Some(Value::Na) => {}
            // A bare name: the variable whose values the colours show.
            Some(Value::Var(name)) => {
                let found = column(ctx, frame, name, ctx.span_of(slot))?;
                if shading(found).is_none() {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`{function}` shows `{}` as categories or on a continuous scale, so `{name}` must be a Bool, Int, Float, Str or Category variable, but it is {}",
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
                    return Err(not_a_colour(
                        ctx, frame, function, slot, aesthetic, written, &why,
                    ));
                }
            },
            Some(_) => {
                return Err(internal(
                    ctx,
                    "a call was given a colour that is not a variable or a string",
                ));
            }
        }
    }
    if let Some(slot) = slots.size {
        match args.get(slot) {
            None | Some(Value::Na) => {}
            // A bare name: the variable whose amounts the areas of the marks show.
            Some(Value::Var(name)) => {
                let found = column(ctx, frame, name, ctx.span_of(slot))?;
                if shading(found) != Some(Shading::Ramp) {
                    let with_labels = if is_numeric(found) && labelled(found) {
                        " with value labels"
                    } else {
                        ""
                    };
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`{function}` shows `size` by area, so `{name}` must be an Int or Float variable without value labels, but it is {}{with_labels}",
                                a_type(found)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(slot), "not an amount")
                        .help("a size shows how much: the area of a mark is in proportion to the value; categories are shown by `color:` or `shape:`"),
                    ));
                }
                mappings.push(PlotMapping {
                    aesthetic: Aesthetic::Size,
                    variable: name.clone(),
                    computed: None,
                });
            }
            // A number: the diameter of every mark.
            Some(Value::Int(_) | Value::Float(_)) => {
                let Some(across) =
                    number_option(ctx, args, slot, "a `size:` that is not a number")?
                else {
                    return Err(internal(ctx, "a `size:` that is a number and is not"));
                };
                if !(across > 0.0 && across.is_finite()) {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`size:` of `{function}` is the diameter of every mark in points of 1/72 inch, above 0, but this is {}",
                                shown(across)
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(slot), "not a size")
                        .help(if function == "plot" {
                            "write a size such as 3 or 6, or the name of a variable to size the marks by; without `size:` a point is 4.5 across"
                        } else {
                            "write a size such as 3 or 6, or the name of a variable to size the marks by; without `size:` a point takes the size the plot gives, or is 4.5 across"
                        }),
                    ));
                }
                constants.push(PlotConstant {
                    aesthetic: Aesthetic::Size,
                    value: PlotSetting::Size(across),
                });
            }
            Some(_) => {
                return Err(internal(
                    ctx,
                    "a call was given a size that is not a variable or a number",
                ));
            }
        }
    }
    if let Some(slot) = slots.shape {
        match args.get(slot) {
            None | Some(Value::Na) => {}
            // A bare name: the variable whose categories the shapes show.
            Some(Value::Var(name)) => {
                let found = column(ctx, frame, name, ctx.span_of(slot))?;
                if shading(found) != Some(Shading::Categories) {
                    let number = shading(found) == Some(Shading::Ramp);
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`{function}` gives each category a shape, so `{name}` must be a Bool, Str or Category variable, or a number with value labels, but it is {}{}",
                                a_type(found),
                                if number { " without value labels" } else { "" }
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(slot), "not categories")
                        .help(if number {
                            format!(
                                "a shape shows a category; a number is shown by `color:` or `size:`, or made categories first, as in `{}.add({name} = category({name}))`",
                                frame.shown_name()
                            )
                        } else {
                            format!(
                                "`{}.describe()` lists the types of the variables",
                                frame.shown_name()
                            )
                        }),
                    ));
                }
                mappings.push(PlotMapping {
                    aesthetic: Aesthetic::Shape,
                    variable: name.clone(),
                    computed: None,
                });
            }
            // A string: one shape for every mark.
            Some(Value::Str(written)) => match shape::parse(written) {
                Ok(name) => constants.push(PlotConstant {
                    aesthetic: Aesthetic::Shape,
                    value: PlotSetting::Shape(name),
                }),
                Err(near) => {
                    return Err(not_a_shape(ctx, frame, function, slot, written, near));
                }
            },
            Some(_) => {
                return Err(internal(
                    ctx,
                    "a call was given a shape that is not a variable or a string",
                ));
            }
        }
    }
    // The variables held for the marks that read them: any type will do.
    for (slot, aesthetic) in [
        (slots.group, Aesthetic::Group),
        (slots.label, Aesthetic::Label),
    ] {
        let Some(slot) = slot else {
            continue;
        };
        match args.get(slot) {
            None | Some(Value::Na) => {}
            Some(Value::Var(name)) => {
                column(ctx, frame, name, ctx.span_of(slot))?;
                mappings.push(PlotMapping {
                    aesthetic,
                    variable: name.clone(),
                    computed: None,
                });
            }
            Some(_) => {
                return Err(internal(
                    ctx,
                    "a call was given a group or a label that is not a variable",
                ));
            }
        }
    }
    // The type of a line: a bare name gives each category of a variable one of the six types,
    // and a string sets one type for every line.
    if let Some(slot) = slots.linetype {
        match args.get(slot) {
            None | Some(Value::Na) => {}
            Some(Value::Var(name)) => {
                // A type of line written without quotation marks reads as a variable.
                if frame.table.column(name).is_none() && LineType::parse(name).is_some() {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0101",
                            format!("no variable `{name}` in `{}`", frame.shown_name()),
                        )
                        .in_file(ctx.script)
                        .primary(
                            ctx.span_of(slot),
                            format!("not a variable of {}", frame.shown_name()),
                        )
                        .help(format!(
                            "`{name}` is a type of line: to draw every line of the layer that way, write it in quotation marks, `linetype: \"{name}\"`"
                        )),
                    ));
                }
                let found = column(ctx, frame, name, ctx.span_of(slot))?;
                if shading(found) != Some(Shading::Categories) {
                    let number = shading(found) == Some(Shading::Ramp);
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`{function}` gives each category a type of line, so `{name}` must be a Bool, Str or Category variable, or a number with value labels, but it is {}{}",
                                a_type(found),
                                if number { " without value labels" } else { "" }
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(slot), "not categories")
                        .help(if number {
                            format!(
                                "a type of line shows a category; make the number categories first, as in `{}.add({name} = category({name}))`",
                                frame.shown_name()
                            )
                        } else {
                            format!(
                                "`{}.describe()` lists the types of the variables",
                                frame.shown_name()
                            )
                        }),
                    ));
                }
                mappings.push(PlotMapping {
                    aesthetic: Aesthetic::Linetype,
                    variable: name.clone(),
                    computed: None,
                });
            }
            Some(Value::Str(written)) => match LineType::parse(written) {
                Some(kind) => constants.push(PlotConstant {
                    aesthetic: Aesthetic::Linetype,
                    value: PlotSetting::Line(kind.name()),
                }),
                None => {
                    // The string may be the variable the script meant to give types by.
                    let help = if frame.table.column(written).is_some() {
                        format!(
                            "a string sets one type for every line; to give each category of `{written}` a type, write it without quotation marks, `linetype: {written}`"
                        )
                    } else {
                        let names: Vec<&str> =
                            LineType::ALL.iter().map(|each| each.name()).collect();
                        let (last, others) = names.split_last().unwrap_or((&"", &[]));
                        format!("a type of line is one of {} and {last}", others.join(", "))
                    };
                    return Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`linetype:` of `{function}` is \"{written}\", which is not a type of line"
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(slot), "not a type of line")
                        .help(help),
                    ));
                }
            },
            Some(_) => {
                return Err(internal(
                    ctx,
                    "a call was given a type of line that is not a variable or a string",
                ));
            }
        }
    }
    Ok((mappings, constants))
}

/// `data.plot(x: Var | Expr, y: Var | Expr, color: Var | Str, fill: Var | Str,
/// size: Var | Float, shape: Var | Str, group: Var, label: Var)`.
pub fn plot(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(DATA) else {
        return Err(internal(ctx, "`plot` reached its body without a frame"));
    };
    let frame: Frame = frame.clone();
    let (mappings, constants) = aesthetics(
        ctx,
        &args,
        &frame,
        "plot",
        &Slots {
            x: Some(X),
            y: Some(Y),
            low: None,
            high: None,
            color: Some(COLOR),
            fill: Some(FILL),
            size: Some(SIZE),
            shape: Some(SHAPE),
            group: Some(GROUP),
            label: Some(LABEL),
            linetype: None,
        },
        false,
    )?;
    Ok(Value::Plot(std::rc::Rc::new(start(
        frame, mappings, constants,
    ))))
}
