//! Package core (tier 0): the language and the types every package shares.

pub mod cases;
pub mod collections;
pub mod display;
pub mod fused;
pub mod lift;
pub mod maths;
pub mod missing;
pub mod operators;
pub mod program;
pub mod runtime;
pub mod special;
pub mod table;
pub mod types;
pub mod value;

use runtime::{Args, CallContext, Fail, Function};
use value::Value;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest, ZeffSource};

/// `print(x, ..., sep: " ")`: each value through the display interface (open decision #95,
/// option A; release 0.8.0, step 8p). Plain values given side by side, a number, a text, a
/// Bool, a date, a time or `NA`, share one line, `sep:` between them; any other value ends
/// the line and prints as its own block, as it always did; every line ends. One value prints
/// exactly as before.
fn print(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let sep = match args.take(1) {
        Some(Value::Str(sep)) => sep,
        _ => " ".to_string(),
    };
    let plain = |value: &Value| {
        matches!(
            value,
            Value::Int(_)
                | Value::Float(_)
                | Value::Str(_)
                | Value::Bool(_)
                | Value::Date(_)
                | Value::DateTime(_)
                | Value::Na
        )
    };
    let first = args.take(0);
    let mut line: Option<String> = None;
    for value in first.into_iter().chain(args.rest) {
        if plain(&value) {
            let shown = display::show(&value);
            let text = shown.strip_suffix('\n').unwrap_or(&shown);
            match line.as_mut() {
                Some(line) => {
                    line.push_str(&sep);
                    line.push_str(text);
                }
                None => line = Some(text.to_string()),
            }
        } else {
            if let Some(line) = line.take() {
                ctx.print(&line);
                ctx.print("\n");
            }
            ctx.print(&display::show(&value));
        }
    }
    if let Some(line) = line {
        ctx.print(&line);
        ctx.print("\n");
    }
    Ok(Value::Nothing)
}

/// The functions of `core` written in Zeff, one `functions/*.zeff` file each (Engine and Project
/// Specification, section 9.1, "Files"), compiled in; the engine parses, checks and lowers them
/// when it starts: `clamp`, `logit`, `inv_logit` (release 0.5.0, step 14) and `assert` (step
/// 15); `fill_missing` was here until release 0.8.0, step 8i, and is now one function of `data`
/// with `fill_forward` (open decision #97); `n_missing` is Rust, since a Zeff signature cannot take a list or a
/// column, or a value of any scalar type, in one parameter (open decision #68).
pub fn zeff_sources() -> Vec<ZeffSource> {
    vec![
        ZeffSource {
            package: "core",
            name: "clamp",
            file: "core/functions/clamp.zeff",
            text: include_str!("../functions/clamp.zeff"),
            cannot: &[],
            also: &[],
        },
        ZeffSource {
            package: "core",
            name: "logit",
            file: "core/functions/logit.zeff",
            text: include_str!("../functions/logit.zeff"),
            cannot: &[
                ("W0204", "said below in the function's own words"),
                ("W0205", "said below in the function's own words"),
            ],
            also: &[
                ("W0204", "`p` is 0 or 1, where the logit is infinite"),
                (
                    "W0205",
                    "`p` is below 0 or above 1, where the logit has no value",
                ),
            ],
        },
        ZeffSource {
            package: "core",
            name: "inv_logit",
            file: "core/functions/inv_logit.zeff",
            text: include_str!("../functions/inv_logit.zeff"),
            cannot: &[],
            also: &[],
        },
        ZeffSource {
            package: "core",
            name: "assert",
            file: "core/functions/assert.zeff",
            text: include_str!("../functions/assert.zeff"),
            cannot: &[],
            also: &[("Z0203", "the condition is `NA`")],
        },
    ]
}

/// The functions of `core` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    let mut all = vec![Function {
        spec: FunctionSpec {
            name: "print",
            package: "core",
            method: false,
            params: vec![
                ParamSpec::required("x", "Any"),
                ParamSpec::option_with_default("sep", "Str", "\" \""),
            ],
            rest: Some(Rest::Values("Any")),
            returns: "Nothing",
            diagnostics: vec![DiagnosticDecl::new(
                "Z0206",
                "the result of print is bound or passed; it has none",
            )],
            help: include_str!("../help/print.md"),
            accuracy: None,
        },
        body: print,
    }];
    all.extend(maths::functions());
    all.extend(special::functions());
    all.extend(missing::functions());
    all.extend(program::functions());
    all.extend(types::functions());
    all.extend(collections::functions());
    all.extend(table::functions());
    all
}
