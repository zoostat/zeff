//! The maths functions of `core` (Language Reference, section 3.7, "Numbers", and 4.1): `abs`,
//! `sign`, `sqrt`, `floor`, `ceil`, `truncate`, `round`, `min`, `max`, `div`, `mod`, `is_finite`
//! and the constants `PI` and `E`, written for one value each; a column is handled row by row by
//! `lift`. The arithmetic itself is the maths kernel's (`zeff_numeric::maths`), one pure-Rust
//! implementation on every target.
//!
//! The type rules of section 3.7: `abs`, `min`, `max`, `div` and `mod` on Ints stay Int, and an
//! overflow is an error (Z0211); `div(a, 0)` and `mod(a, 0)` with Ints are errors (Z0212), with
//! Floats they follow `/`; `sqrt`, `round`, `floor`, `ceil` and `truncate` give Float; `sign`
//! gives an Int; `NA` in gives `NA` out, except for `is_finite`, which is false.

use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_numeric::maths::{self, RoundingMode};
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

/// The number in an argument: Int, Float, or `NA`; the checker admits nothing else.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Number {
    Int(i64),
    Float(f64),
    Na,
}

fn number(value: Option<&Value>) -> Number {
    match value {
        Some(Value::Int(i)) => Number::Int(*i),
        Some(Value::Float(x)) => Number::Float(*x),
        _ => Number::Na,
    }
}

fn as_float(n: Number) -> Option<f64> {
    match n {
        Number::Int(i) => Some(i as f64),
        Number::Float(x) => Some(x),
        Number::Na => None,
    }
}

/// A Float result: `NA` for not-a-number, as section 4.1 has it.
fn float(x: f64) -> Value {
    if x.is_nan() {
        Value::Na
    } else {
        Value::Float(x)
    }
}

/// Z0211 at the argument `at`.
fn overflow(ctx: &CallContext<'_>, at: usize, operation: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z0211",
            format!("Int overflow: {operation} does not fit in 64 bits"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(at), "the operation")
        .help("Ints are 64-bit; a Float (`1.0 * x`) holds larger magnitudes with 53 bits of precision"),
    )
}

/// W0205: an operation with no value gave `NA`; the wording names the cause and not the value,
/// so that a column's rows count under one line.
fn gave_na(ctx: &mut CallContext<'_>, cause: &str, help: &str) {
    ctx.message(Diagnostic::warning("W0205", format!("{cause} gave NA")).help(help));
}

fn abs(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match number(args.get(0)) {
        Number::Int(i) => match i.checked_abs() {
            Some(a) => Value::Int(a),
            None => return Err(overflow(ctx, 0, &format!("abs({i})"))),
        },
        Number::Float(x) => Value::Float(x.abs()),
        Number::Na => Value::Na,
    })
}

fn sign(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match number(args.get(0)) {
        Number::Int(i) => Value::Int(i.signum()),
        Number::Float(x) if x.is_nan() => Value::Na,
        Number::Float(x) => Value::Int(maths::sign(x) as i64),
        Number::Na => Value::Na,
    })
}

fn sqrt(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(x) = as_float(number(args.get(0))) else {
        return Ok(Value::Na);
    };
    if x < 0.0 {
        gave_na(
            ctx,
            "sqrt of a negative number",
            "a negative number has no real square root; `where(x >= 0)` keeps the rows that have one",
        );
        return Ok(Value::Na);
    }
    Ok(float(maths::sqrt(x)))
}

fn floor(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0))).map_or(Value::Na, |x| float(x.floor())))
}

fn ceil(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0))).map_or(Value::Na, |x| float(x.ceil())))
}

fn truncate(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0))).map_or(Value::Na, |x| float(x.trunc())))
}

/// `round(x, digits: Int = 0, sig: Int, mode: even | away = even)`.
fn round(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let digits = args.get(1);
    let sig = args.get(2);
    if let (Some(Value::Int(_)), Some(Value::Int(_))) = (digits, sig) {
        return Err(fail(
            Diagnostic::error(
                "Z0410",
                "`round` takes `digits:` or `sig:`, not both",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "`sig:` beside `digits:`")
            .help("`digits:` counts decimal places and `sig:` significant digits; keep the one you mean"),
        ));
    }
    let mode = match args.get(3) {
        Some(Value::Word(w)) if w == "away" => RoundingMode::Away,
        _ => RoundingMode::Even,
    };
    let Some(x) = as_float(number(args.get(0))) else {
        return Ok(Value::Na);
    };
    if let Some(Value::Int(sig)) = sig {
        if *sig < 1 {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`sig:` of `round` must be at least 1, but this is {sig}"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "not a count of significant digits")
                .help("`sig: 3` keeps three significant digits: 123456 becomes 123000"),
            ));
        }
        let sig = u32::try_from(*sig).unwrap_or(u32::MAX);
        return Ok(float(maths::round_significant(x, sig, mode)));
    }
    let digits = match digits {
        Some(Value::Int(d)) => {
            i32::try_from(*d).unwrap_or(if *d < 0 { i32::MIN } else { i32::MAX })
        }
        _ => 0,
    };
    Ok(float(maths::round(x, digits, mode)))
}

/// `min(a, b, ...)` and `max(a, b, ...)`, row by row: Int when every value is an Int; `NA` when
/// any value is `NA`.
fn extreme(args: &Args, smallest: bool) -> Value {
    let numbers: Vec<Number> = args
        .values
        .iter()
        .map(|v| number(v.as_ref()))
        .chain(args.rest.iter().map(|v| number(Some(v))))
        .collect();
    if numbers.iter().any(|n| matches!(n, Number::Na)) {
        return Value::Na;
    }
    if numbers.iter().all(|n| matches!(n, Number::Int(_))) {
        let ints = numbers.iter().filter_map(|n| match n {
            Number::Int(i) => Some(*i),
            _ => None,
        });
        let chosen = if smallest { ints.min() } else { ints.max() };
        return chosen.map_or(Value::Na, Value::Int);
    }
    let floats = numbers.iter().filter_map(|n| as_float(*n));
    // IEEE min and max; a NaN never reaches here (NA is caught above).
    let chosen = if smallest {
        floats.fold(f64::INFINITY, f64::min)
    } else {
        floats.fold(f64::NEG_INFINITY, f64::max)
    };
    Value::Float(chosen)
}

fn min(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(extreme(&args, true))
}

fn max(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(extreme(&args, false))
}

/// Z0212: a whole-number division by zero.
fn zero_divisor(ctx: &CallContext<'_>, name: &str, a: i64) -> Fail {
    fail(
        Diagnostic::error(
            "Z0212",
            format!("division by zero: `{name}({a}, 0)` has no value"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(1), "a divisor of zero")
        .help("test the divisor first, as in `b != 0 && div(a, b) > 1`; with Floats, `div` and `mod` follow `/` and give Inf or NA with a warning"),
    )
}

/// `div(a, b)` and `mod(a, b)` on Ints or Floats (section 3.7).
fn division(ctx: &mut CallContext<'_>, args: &Args, remainder: bool) -> Result<Value, Fail> {
    let name = if remainder { "mod" } else { "div" };
    let (a, b) = (number(args.get(0)), number(args.get(1)));
    match (a, b) {
        (Number::Na, _) | (_, Number::Na) => Ok(Value::Na),
        (Number::Int(a), Number::Int(b)) => {
            if b == 0 {
                return Err(zero_divisor(ctx, name, a));
            }
            let result = if remainder {
                maths::mod_int(a, b)
            } else {
                maths::div_int(a, b)
            };
            result
                .map(Value::Int)
                .ok_or_else(|| overflow(ctx, 0, &format!("{name}({a}, {b})")))
        }
        (a, b) => {
            let (a, b) = (
                as_float(a).unwrap_or(f64::NAN),
                as_float(b).unwrap_or(f64::NAN),
            );
            if remainder {
                let r = maths::mod_float(a, b);
                if r.is_nan() {
                    let cause = if b == 0.0 {
                        "mod by zero"
                    } else {
                        "mod of an infinite value"
                    };
                    gave_na(
                        ctx,
                        cause,
                        "the remainder has no value here; `where(b != 0)` keeps the rows with a divisor",
                    );
                }
                Ok(float(r))
            } else {
                let q = maths::div_float(a, b);
                if q.is_nan() {
                    gave_na(
                        ctx,
                        "div(0, 0)",
                        "the quotient has no value here; `where(b != 0)` keeps the rows with a divisor",
                    );
                } else if q.is_infinite() && a.is_finite() {
                    ctx.message(
                        Diagnostic::warning("W0204", "division by zero gave Inf or -Inf")
                            .help("`where(b != 0)` keeps the rows with a divisor; the infinities stay otherwise"),
                    );
                }
                Ok(float(q))
            }
        }
    }
}

fn div(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    division(ctx, &args, false)
}

fn modulo(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    division(ctx, &args, true)
}

fn is_finite(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(Value::Bool(match number(args.get(0)) {
        Number::Int(_) => true,
        Number::Float(x) => x.is_finite(),
        Number::Na => false,
    }))
}

fn pi(_ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Ok(Value::Float(std::f64::consts::PI))
}

fn e(_ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Ok(Value::Float(std::f64::consts::E))
}

/// A function of one number.
fn unary(
    name: &'static str,
    returns: &'static str,
    help: &'static str,
    body: crate::runtime::Body,
    diagnostics: Vec<DiagnosticDecl>,
) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "core",
            method: false,
            params: vec![ParamSpec::required("x", "Float")],
            rest: None,
            returns,
            diagnostics,
            help,
            accuracy: None,
        },
        body,
    }
}

fn na_on_overflow() -> DiagnosticDecl {
    DiagnosticDecl::new("Z0211", "an Int result does not fit in 64 bits")
}

/// The maths functions and constants of `core` in this release.
pub fn functions() -> Vec<Function> {
    vec![
        unary(
            "abs",
            "Number",
            include_str!("../help/abs.md"),
            abs,
            vec![na_on_overflow()],
        ),
        unary("sign", "Int", include_str!("../help/sign.md"), sign, vec![]),
        unary(
            "sqrt",
            "Float",
            include_str!("../help/sqrt.md"),
            sqrt,
            vec![DiagnosticDecl::new(
                "W0205",
                "a negative number has no real square root: the result is NA",
            )],
        ),
        unary(
            "floor",
            "Float",
            include_str!("../help/floor.md"),
            floor,
            vec![],
        ),
        unary(
            "ceil",
            "Float",
            include_str!("../help/ceil.md"),
            ceil,
            vec![],
        ),
        unary(
            "truncate",
            "Float",
            include_str!("../help/truncate.md"),
            truncate,
            vec![],
        ),
        Function {
            spec: FunctionSpec {
                name: "round",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Float"),
                    ParamSpec::option_with_default("digits", "Int", "0"),
                    ParamSpec::option("sig", "Int"),
                    ParamSpec::option_with_default("mode", "even | away", "even"),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0401", "sig: is less than 1"),
                    DiagnosticDecl::new("Z0410", "digits: and sig: are both given"),
                ],
                help: include_str!("../help/round.md"),
                accuracy: None,
            },
            body: round,
        },
        Function {
            spec: FunctionSpec {
                name: "min",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("a", "Float"),
                    ParamSpec::required("b", "Float"),
                ],
                rest: Some(Rest::Values("Float")),
                returns: "Number",
                diagnostics: vec![],
                help: include_str!("../help/min.md"),
                accuracy: None,
            },
            body: min,
        },
        Function {
            spec: FunctionSpec {
                name: "max",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("a", "Float"),
                    ParamSpec::required("b", "Float"),
                ],
                rest: Some(Rest::Values("Float")),
                returns: "Number",
                diagnostics: vec![],
                help: include_str!("../help/max.md"),
                accuracy: None,
            },
            body: max,
        },
        Function {
            spec: FunctionSpec {
                name: "div",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("a", "Float"),
                    ParamSpec::required("b", "Float"),
                ],
                rest: None,
                returns: "Number",
                diagnostics: vec![
                    DiagnosticDecl::new("W0204", "a Float divided by zero gave Inf or -Inf"),
                    DiagnosticDecl::new("W0205", "0 / 0 with Floats gave NA"),
                    na_on_overflow(),
                    DiagnosticDecl::new("Z0212", "a whole-number division by zero"),
                ],
                help: include_str!("../help/div.md"),
                accuracy: None,
            },
            body: div,
        },
        Function {
            spec: FunctionSpec {
                name: "mod",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("a", "Float"),
                    ParamSpec::required("b", "Float"),
                ],
                rest: None,
                returns: "Number",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0205",
                        "a Float remainder by zero, or of an infinite value, gave NA",
                    ),
                    DiagnosticDecl::new("Z0212", "a whole-number division by zero"),
                ],
                help: include_str!("../help/mod.md"),
                accuracy: None,
            },
            body: modulo,
        },
        unary(
            "is_finite",
            "Bool",
            include_str!("../help/is_finite.md"),
            is_finite,
            vec![],
        ),
        Function {
            spec: FunctionSpec {
                name: "PI",
                package: "core",
                method: false,
                params: vec![],
                rest: None,
                returns: "Float",
                diagnostics: vec![],
                help: include_str!("../help/PI.md"),
                accuracy: None,
            },
            body: pi,
        },
        Function {
            spec: FunctionSpec {
                name: "E",
                package: "core",
                method: false,
                params: vec![],
                rest: None,
                returns: "Float",
                diagnostics: vec![],
                help: include_str!("../help/E.md"),
                accuracy: None,
            },
            body: e,
        },
    ]
}
