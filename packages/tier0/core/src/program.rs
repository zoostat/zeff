//! The program functions of `core` that a script uses to speak for itself (Language Reference,
//! section 5.4): `warn(message)` prints a warning under the statement's output and carries on,
//! `note(message)` a note, each with a warning's code when `code:` gives one (open decision
//! #70); `error(message)` ends the script with Z0001; `name_of(v)` is a variable's name as
//! text. `print` is in `lib.rs`.

use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The text of a `message` argument; `NA` reads as the word.
fn message(args: &Args) -> String {
    match args.get(0) {
        Some(Value::Str(s)) => s.clone(),
        _ => "NA".to_string(),
    }
}

/// The `code:` of `warn` and `note`, when given: a warning's code, `W` and four digits.
/// Whether the engine raises it is for the engine to say, which knows its pages; an error's
/// code is refused here (Z0401), since a warning cannot carry one.
fn code_of(ctx: &CallContext<'_>, args: &Args, name: &str) -> Result<Option<String>, Fail> {
    let code = match args.get(1) {
        None | Some(Value::Na) => return Ok(None),
        Some(Value::Str(code)) => code.clone(),
        Some(other) => other.type_name().to_string(),
    };
    let warnings =
        code.len() == 5 && code.starts_with('W') && code[1..].bytes().all(|b| b.is_ascii_digit());
    if warnings {
        return Ok(Some(code));
    }
    let label = if code.starts_with('Z') {
        "an error's code"
    } else {
        "not a warning's code"
    };
    Err(fail(
        Diagnostic::error(
            "Z0401",
            format!("`code` of `{name}` must be a warning's code, but this is `{code}`"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(1), label)
        .help("a warning's code is `W` and four digits, as in `code: W0205`; `zeff help W0205` explains a code"),
    ))
}

fn warn(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let message = message(&args);
    ctx.message(match code_of(ctx, &args, "warn")? {
        Some(code) => Diagnostic::warning(&code, message),
        None => Diagnostic::plain_warning(message),
    });
    Ok(Value::Nothing)
}

fn note(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let message = message(&args);
    ctx.message(match code_of(ctx, &args, "note")? {
        Some(code) => Diagnostic::note(&code, message),
        None => Diagnostic::plain_note(message),
    });
    Ok(Value::Nothing)
}

/// `name_of(v)`: the variable's name as text; `NA` for `NA`.
fn name_of(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match args.get(0) {
        Some(Value::Var(name)) => Ok(Value::Str(name.clone())),
        None | Some(Value::Na) => Ok(Value::Na),
        Some(other) => Err(fail(
            Diagnostic::error(
                "Z9001",
                format!(
                    "internal error: this is a bug in Zeff, not in your script (`name_of` was given {}, which the check refuses)",
                    other.type_name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        )),
    }
}

fn error(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error("Z0001", message(&args))
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the script stops here"),
    ))
}

/// `attempt(expr)` never reaches a body: the checker lowers it to its own node and the
/// interpreter evaluates the expression under a catch. A call that arrives here is a bug.
fn iterate(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error(
            "Z9001",
            "internal error: this is a bug in Zeff, not in your script (`iterate` reached a body)",
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    ))
}

fn attempt(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error(
            "Z9001",
            "internal error: this is a bug in Zeff, not in your script (`attempt` reached a body)",
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    ))
}

/// `seed(n)`: the run's seed, set once per script before any draw (Engine and Project
/// Specification, section 8.3, point 3; the catalogue's row 63). A second `seed`, or one after a
/// statement has drawn, is Z0111: the draws before it would not be reproduced by the seed printed.
fn seed(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let n = match args.get(0) {
        Some(Value::Int(n)) => *n,
        _ => {
            return Err(fail(
                Diagnostic::error("Z0401", "`n` of `seed` must be an Int, but this is NA")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(0), "not a seed"),
            ));
        }
    };
    if ctx.random.used {
        let chosen = ctx.random.seed.unwrap_or_default();
        return Err(fail(
            Diagnostic::error(
                "Z0111",
                "`seed` comes after a statement that has already drawn random numbers",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "too late to set the seed")
            .help(format!(
                "put `seed({n})` at the top of the script, before the first draw; the draws so far used the seed {chosen}"
            )),
        ));
    }
    if ctx.random.seed.is_some() {
        return Err(fail(
            Diagnostic::error("Z0111", "`seed` is set once per script, and it is set already")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "a second seed")
                .help("one seed makes the whole run repeatable; a `seed:` given to one `draw` has a stream of its own"),
        ));
    }
    ctx.random.seed = Some(n);
    ctx.random.chosen = false;
    Ok(Value::Nothing)
}

/// The program functions of `core` in this release, `print` apart.
pub fn functions() -> Vec<Function> {
    vec![
        // `attempt(expr)` is run by the interpreter, which evaluates the expression under a
        // catch (Language Reference, section 5.4); the entry here carries its page and signature.
        Function {
            spec: FunctionSpec {
                name: "attempt",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("expr", "Any")],
                rest: None,
                returns: "Any",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0220",
                        "the failed value is used outside a list: as an argument, in arithmetic, in a frame or as a model input",
                    ),
                    DiagnosticDecl::new(
                        "W0209",
                        "a method is called on a binding that may hold a failed value, without a guard",
                    ),
                ],
                help: include_str!("../help/attempt.md"),
                accuracy: None,
            },
            body: attempt,
        },
        // `iterate(start:, step:, done:, ...)` is run by the interpreter, which calls the step
        // and the stopping function it is given (Language Reference, section 5.5); the entry here
        // carries its page and signature.
        Function {
            spec: FunctionSpec {
                name: "iterate",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::option("start", "Record"),
                    ParamSpec::option("step", "Fn(Record, ..) -> Record"),
                    ParamSpec::option("done", "Fn(Record, Record) -> Bool"),
                    ParamSpec::option("args", "Record"),
                    ParamSpec::option_with_default("max", "Int", "1000"),
                    ParamSpec::option_with_default("keep", "Bool", "false"),
                ],
                rest: None,
                returns: "Record",
                diagnostics: vec![
                    DiagnosticDecl::note("the number of passes made"),
                    DiagnosticDecl::new(
                        "W0805",
                        "the passes stopped at max: without done becoming true",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "max: below 1, or a step whose record gains or loses a field of the state",
                    ),
                    DiagnosticDecl::new("Z0203", "done gave NA where true or false was needed"),
                ],
                help: include_str!("../help/iterate.md"),
                accuracy: None,
            },
            body: iterate,
        },
        Function {
            spec: FunctionSpec {
                name: "seed",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("n", "Int")],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0111", "a second seed, or one after a draw"),
                    DiagnosticDecl::new(
                        "Z0206",
                        "the result of seed is bound or passed; it has none",
                    ),
                ],
                help: include_str!("../help/seed.md"),
                accuracy: None,
            },
            body: seed,
        },
        Function {
            spec: FunctionSpec {
                name: "warn",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("message", "Str"),
                    ParamSpec::option("code", "Code"),
                ],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the script's own warning, printed with its line and with the code `code:` gives it",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "code: is not a warning's code, or is one the engine does not raise",
                    ),
                    DiagnosticDecl::new(
                        "Z0206",
                        "the result of warn is bound or passed; it has none",
                    ),
                ],
                help: include_str!("../help/warn.md"),
                accuracy: None,
            },
            body: warn,
        },
        Function {
            spec: FunctionSpec {
                name: "note",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("message", "Str"),
                    ParamSpec::option("code", "Code"),
                ],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the script's own note, printed with its line and with the code `code:` gives it",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "code: is not a warning's code, or is one the engine does not raise",
                    ),
                    DiagnosticDecl::new(
                        "Z0206",
                        "the result of note is bound or passed; it has none",
                    ),
                ],
                help: include_str!("../help/note.md"),
                accuracy: None,
            },
            body: note,
        },
        Function {
            spec: FunctionSpec {
                name: "name_of",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("v", "Var")],
                rest: None,
                returns: "Str",
                diagnostics: Vec::new(),
                help: include_str!("../help/name_of.md"),
                accuracy: None,
            },
            body: name_of,
        },
        Function {
            spec: FunctionSpec {
                name: "error",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("message", "Str")],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0001",
                    "the script stopped itself with the message given",
                )],
                help: include_str!("../help/error.md"),
                accuracy: None,
            },
            body: error,
        },
    ]
}
