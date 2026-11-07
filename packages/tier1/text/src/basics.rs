//! The text functions of release 0.7.0, step 5 (catalogue rows 83 to 85, 87, 88 and 92 to 94;
//! Language Reference, section 3.8, "Text"): `upper` and `lower` by Unicode simple case mapping,
//! one scalar value to one; `trim` of Unicode white space; `starts_with` and `ends_with`;
//! `slice` and `pad` counting Unicode scalar values from 1; and `join` of a list of texts. Each
//! works on one text and applies row by row to a column through the lifting rule (section 3.7,
//! "Columns and scalars"); `NA` gives `NA`.

use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// Z9001: a body reached with an argument the checker would have refused.
fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// The text at argument `i`, `None` for `NA` or an argument left out.
fn text_at<'a>(ctx: &CallContext<'_>, args: &'a Args, i: usize) -> Result<Option<&'a str>, Fail> {
    match args.get(i) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Str(s)) => Ok(Some(s.as_str())),
        Some(_) => Err(internal(ctx, "an argument that is not text")),
    }
}

/// The whole number at argument `i`, `None` for `NA` or an argument left out.
fn int_at(ctx: &CallContext<'_>, args: &Args, i: usize) -> Result<Option<i64>, Fail> {
    match args.get(i) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Int(n)) => Ok(Some(*n)),
        Some(_) => Err(internal(ctx, "an argument that is not an Int")),
    }
}

/// Unicode simple case mapping: the one scalar value a character maps to, or the character
/// itself where the full mapping gives more than one (`ß`, `ﬁ`), so that the length of a text
/// never changes (section 3.8).
fn simple_upper(c: char) -> char {
    let mut mapped = c.to_uppercase();
    match (mapped.next(), mapped.next()) {
        (Some(one), None) => one,
        _ => c,
    }
}

fn simple_lower(c: char) -> char {
    let mut mapped = c.to_lowercase();
    match (mapped.next(), mapped.next()) {
        (Some(one), None) => one,
        _ => c,
    }
}

fn upper(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match text_at(ctx, &args, 0)? {
        None => Value::Na,
        Some(s) => Value::Str(s.chars().map(simple_upper).collect()),
    })
}

fn lower(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match text_at(ctx, &args, 0)? {
        None => Value::Na,
        Some(s) => Value::Str(s.chars().map(simple_lower).collect()),
    })
}

/// `trim(s)`: the text without its leading and trailing Unicode white space (the characters
/// with the `White_Space` property, which `char::is_whitespace` tests).
fn trim(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match text_at(ctx, &args, 0)? {
        None => Value::Na,
        Some(s) => Value::Str(s.trim().to_string()),
    })
}

fn starts_with(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match (text_at(ctx, &args, 0)?, text_at(ctx, &args, 1)?) {
        (Some(s), Some(prefix)) => Value::Bool(s.starts_with(prefix)),
        _ => Value::Na,
    })
}

fn ends_with(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match (text_at(ctx, &args, 0)?, text_at(ctx, &args, 1)?) {
        (Some(s), Some(suffix)) => Value::Bool(s.ends_with(suffix)),
        _ => Value::Na,
    })
}

/// `slice(s, start:, len:)`: `len` scalar values from position `start`, counted from 1; what
/// is there when the text ends first. Z0406 without `start:` or `len:`, Z0401 for a start
/// below 1 or a negative length; `NA` in any argument gives `NA`.
fn slice(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    for (i, name) in [(1, "start"), (2, "len")] {
        if args.get(i).is_none() {
            return Err(fail(
                Diagnostic::error("Z0406", format!("`slice` needs `{name}:`"))
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, format!("`{name}:` is missing"))
                    .help("usage: `slice(s, start: 1, len: 3)` for the first three characters"),
            ));
        }
    }
    let text = text_at(ctx, &args, 0)?;
    let start = int_at(ctx, &args, 1)?;
    let len = int_at(ctx, &args, 2)?;
    if let Some(start) = start
        && start < 1
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`start:` of `slice` counts positions from 1, but this is {start}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "before the first character")
            .help("the first character is at position 1, as in R's `substr` and Stata's `substr`"),
        ));
    }
    if let Some(len) = len
        && len < 0
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`len:` of `slice` is a number of characters, but this is {len}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "a negative length")
            .help("`len: 0` gives the empty text; a length past the end gives what is there"),
        ));
    }
    Ok(match (text, start, len) {
        (Some(s), Some(start), Some(len)) => Value::Str(
            s.chars()
                .skip((start - 1) as usize)
                .take(len as usize)
                .collect(),
        ),
        _ => Value::Na,
    })
}

/// `pad(s, width:, fill: " ")`: the text with `fill` repeated on its left until it is `width`
/// scalar values long; a text already that long or longer is returned as it is. Z0406 without
/// `width:`, Z0401 for a negative width or a fill that is not one character.
fn pad(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    if args.get(1).is_none() {
        return Err(fail(
            Diagnostic::error("Z0406", "`pad` needs `width:`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "`width:` is missing")
                .help("usage: `pad(s, width: 8)` for eight characters, spaces on the left"),
        ));
    }
    let text = text_at(ctx, &args, 0)?;
    let width = int_at(ctx, &args, 1)?;
    let fill = text_at(ctx, &args, 2)?.unwrap_or(" ");
    if let Some(width) = width
        && width < 0
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`width:` of `pad` is a number of characters, but this is {width}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "a negative width")
            .help("a width below the text's length leaves the text as it is"),
        ));
    }
    let mut fill_chars = fill.chars();
    let fill_char = match (fill_chars.next(), fill_chars.next()) {
        (Some(c), None) => c,
        _ => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`fill:` of `pad` is one character, but this is {}",
                        if fill.is_empty() {
                            "empty".to_string()
                        } else {
                            format!("{} characters", fill.chars().count())
                        }
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "not one character")
                .help("`fill: \"0\"` pads a number with zeros, `fill: \".\"` leads the eye across a table"),
            ));
        }
    };
    Ok(match (text, width) {
        (Some(s), Some(width)) => {
            let have = s.chars().count();
            let want = width as usize;
            if have >= want {
                Value::Str(s.to_string())
            } else {
                let mut out: String = std::iter::repeat_n(fill_char, want - have).collect();
                out.push_str(s);
                Value::Str(out)
            }
        }
        _ => Value::Na,
    })
}

/// `join(list, sep: "")`: the texts of a list put together with `sep` between them; an `NA`
/// item is written as `NA`, as R's `paste` writes it, and an empty list gives the empty text.
fn join(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::List(items)) = args.get(0) else {
        return Err(internal(ctx, "a `list` that is not a list"));
    };
    let sep = text_at(ctx, &args, 1)?.unwrap_or("");
    let mut parts: Vec<&str> = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::Str(s) => parts.push(s.as_str()),
            Value::Na => parts.push("NA"),
            _ => return Err(internal(ctx, "an item that is not text")),
        }
    }
    Ok(Value::Str(parts.join(sep)))
}

/// The text functions of this step.
pub fn functions() -> Vec<Function> {
    let one_text = |name: &'static str, help: &'static str, body| Function {
        spec: FunctionSpec {
            name,
            package: "text",
            method: false,
            params: vec![ParamSpec::required("s", "Str")],
            rest: None,
            returns: "Str",
            diagnostics: Vec::new(),
            help,
            accuracy: None,
        },
        body,
    };
    let affix = |name: &'static str, second: &'static str, help: &'static str, body| Function {
        spec: FunctionSpec {
            name,
            package: "text",
            method: false,
            params: vec![
                ParamSpec::required("s", "Str"),
                ParamSpec::required(second, "Str"),
            ],
            rest: None,
            returns: "Bool",
            diagnostics: Vec::new(),
            help,
            accuracy: None,
        },
        body,
    };
    vec![
        one_text("upper", include_str!("../help/upper.md"), upper),
        one_text("lower", include_str!("../help/lower.md"), lower),
        one_text("trim", include_str!("../help/trim.md"), trim),
        affix(
            "starts_with",
            "prefix",
            include_str!("../help/starts_with.md"),
            starts_with,
        ),
        affix(
            "ends_with",
            "suffix",
            include_str!("../help/ends_with.md"),
            ends_with,
        ),
        Function {
            spec: FunctionSpec {
                name: "slice",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::option("start", "Int"),
                    ParamSpec::option("len", "Int"),
                ],
                rest: None,
                returns: "Str",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0406", "start: or len: missing"),
                    DiagnosticDecl::new("Z0401", "start: below 1, or len: below 0"),
                ],
                help: include_str!("../help/slice.md"),
                accuracy: None,
            },
            body: slice,
        },
        Function {
            spec: FunctionSpec {
                name: "pad",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::option("width", "Int"),
                    ParamSpec::option_with_default("fill", "Str", "\" \""),
                ],
                rest: None,
                returns: "Str",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0406", "width: missing"),
                    DiagnosticDecl::new("Z0401", "width: below 0, or fill: not one character"),
                ],
                help: include_str!("../help/pad.md"),
                accuracy: None,
            },
            body: pad,
        },
        Function {
            spec: FunctionSpec {
                name: "join",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("list", "List<Str>"),
                    ParamSpec::option_with_default("sep", "Str", "\"\""),
                ],
                rest: None,
                returns: "Str",
                diagnostics: Vec::new(),
                help: include_str!("../help/join.md"),
                accuracy: None,
            },
            body: join,
        },
    ]
}
