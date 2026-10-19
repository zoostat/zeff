//! The maths functions of `core` (Language Reference, section 3.7, "Numbers", and 4.1): `abs`,
//! `sign`, `sqrt`, `floor`, `ceil`, `truncate`, `round`, `min`, `max`, `div`, `mod`, `is_finite`,
//! the constants `PI` and `E`, the exponential and the logarithms, and the trigonometric
//! functions, written for one value each; a column is handled row by row by
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
pub(crate) enum Number {
    Int(i64),
    Float(f64),
    Na,
}

pub(crate) fn number(value: Option<&Value>) -> Number {
    match value {
        Some(Value::Int(i)) => Number::Int(*i),
        Some(Value::Float(x)) => Number::Float(*x),
        _ => Number::Na,
    }
}

pub(crate) fn as_float(n: Number) -> Option<f64> {
    match n {
        Number::Int(i) => Some(i as f64),
        Number::Float(x) => Some(x),
        Number::Na => None,
    }
}

/// A Float result: `NA` for not-a-number, as section 4.1 has it.
pub(crate) fn float(x: f64) -> Value {
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
pub(crate) fn gave_na(ctx: &mut CallContext<'_>, cause: &str, help: &str) {
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

/// W0204: an operation gave an infinity; the wording names the cause, as `gave_na` does.
pub(crate) fn gave_inf(ctx: &mut CallContext<'_>, cause: &str, sign_negative: bool, help: &str) {
    let inf = if sign_negative { "-Inf" } else { "Inf" };
    ctx.message(Diagnostic::warning("W0204", format!("{cause} gave {inf}")).help(help));
}

const LOG_HELP: &str =
    "`log1p(x)` is log(1 + x) and is finite at zero; or keep the rows with `where(x > 0)`";

/// The logarithm rules of section 4.1 for `x` under `name`: zero gives an infinity with W0204 and a
/// negative number gives `NA` with W0205. `value` is the logarithm already computed.
fn logarithm(ctx: &mut CallContext<'_>, name: &str, x: f64, value: f64) -> Value {
    if x == 0.0 {
        gave_inf(
            ctx,
            &format!("{name} of zero"),
            value.is_sign_negative(),
            LOG_HELP,
        );
        return Value::Float(value);
    }
    if x < 0.0 || value.is_nan() {
        gave_na(
            ctx,
            &format!("{name} of a negative number"),
            "a negative number has no real logarithm; `where(x > 0)` keeps the rows that have one",
        );
        return Value::Na;
    }
    float(value)
}

fn exp(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0))).map_or(Value::Na, |x| float(maths::exp(x))))
}

fn expm1(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0))).map_or(Value::Na, |x| float(maths::expm1(x))))
}

/// `log(x, base: Float = E)`.
fn log(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(x) = as_float(number(args.get(0))) else {
        return Ok(Value::Na);
    };
    let base = match args.get(1) {
        None => None,
        Some(Value::Na) => return Ok(Value::Na),
        other => as_float(number(other)),
    };
    match base {
        None => Ok(logarithm(ctx, "log", x, maths::log(x))),
        Some(base) if base <= 0.0 || base == 1.0 || base.is_nan() => {
            gave_na(
                ctx,
                "log to a base that is not positive or is 1",
                "a logarithm needs a positive base other than 1; `base: 2`, `base: 10` and `E` are the usual ones",
            );
            Ok(Value::Na)
        }
        Some(base) => Ok(logarithm(ctx, "log", x, maths::log_base(x, base))),
    }
}

fn log2(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0)))
        .map_or(Value::Na, |x| logarithm(ctx, "log2", x, maths::log2(x))))
}

fn log10(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0)))
        .map_or(Value::Na, |x| logarithm(ctx, "log10", x, maths::log10(x))))
}

fn log1p(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(x) = as_float(number(args.get(0))) else {
        return Ok(Value::Na);
    };
    if x == -1.0 {
        gave_inf(
            ctx,
            "log1p of -1",
            true,
            "log1p(x) is log(1 + x), which has no value at -1; `where(x > -1)` keeps the rows that have one",
        );
        return Ok(Value::Float(f64::NEG_INFINITY));
    }
    if x < -1.0 {
        gave_na(
            ctx,
            "log1p below -1",
            "log1p(x) is log(1 + x), which has no real value below -1; `where(x > -1)` keeps the rows that have one",
        );
        return Ok(Value::Na);
    }
    Ok(float(maths::log1p(x)))
}

/// A trigonometric function, which has no value at an infinity (W0205).
fn trigonometric(ctx: &mut CallContext<'_>, name: &str, args: &Args, f: fn(f64) -> f64) -> Value {
    let Some(x) = as_float(number(args.get(0))) else {
        return Value::Na;
    };
    if x.is_infinite() {
        gave_na(
            ctx,
            &format!("{name} of an infinite value"),
            "an angle must be finite; `where(is_finite(x))` keeps the rows that are",
        );
        return Value::Na;
    }
    float(f(x))
}

fn sin(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(trigonometric(ctx, "sin", &args, maths::sin))
}

fn cos(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(trigonometric(ctx, "cos", &args, maths::cos))
}

fn tan(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(trigonometric(ctx, "tan", &args, maths::tan))
}

/// An inverse sine or cosine, defined on [-1, 1] only (W0205 outside).
fn inverse(ctx: &mut CallContext<'_>, name: &str, args: &Args, f: fn(f64) -> f64) -> Value {
    let Some(x) = as_float(number(args.get(0))) else {
        return Value::Na;
    };
    if !(-1.0..=1.0).contains(&x) {
        gave_na(
            ctx,
            &format!("{name} outside [-1, 1]"),
            "a sine or cosine lies between -1 and 1, so its inverse has no value elsewhere; `where(abs(x) <= 1)` keeps the rows that have one",
        );
        return Value::Na;
    }
    float(f(x))
}

fn asin(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(inverse(ctx, "asin", &args, maths::asin))
}

fn acos(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(inverse(ctx, "acos", &args, maths::acos))
}

fn atan(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(as_float(number(args.get(0))).map_or(Value::Na, |x| float(maths::atan(x))))
}

fn atan2(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(y), Some(x)) = (as_float(number(args.get(0))), as_float(number(args.get(1)))) else {
        return Ok(Value::Na);
    };
    Ok(float(maths::atan2(y, x)))
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
        unary("exp", "Float", include_str!("../help/exp.md"), exp, vec![]),
        Function {
            spec: FunctionSpec {
                name: "log",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Float"),
                    ParamSpec::option_with_default("base", "Float", "E"),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0204",
                        "log of zero gave -Inf (or Inf for a base below 1)",
                    ),
                    DiagnosticDecl::new(
                        "W0205",
                        "log of a negative number, or to a base that is not positive or is 1, gave NA",
                    ),
                ],
                help: include_str!("../help/log.md"),
                accuracy: None,
            },
            body: log,
        },
        unary(
            "log2",
            "Float",
            include_str!("../help/log2.md"),
            log2,
            vec![
                DiagnosticDecl::new("W0204", "log2 of zero gave -Inf"),
                DiagnosticDecl::new("W0205", "log2 of a negative number gave NA"),
            ],
        ),
        unary(
            "log10",
            "Float",
            include_str!("../help/log10.md"),
            log10,
            vec![
                DiagnosticDecl::new("W0204", "log10 of zero gave -Inf"),
                DiagnosticDecl::new("W0205", "log10 of a negative number gave NA"),
            ],
        ),
        unary(
            "log1p",
            "Float",
            include_str!("../help/log1p.md"),
            log1p,
            vec![
                DiagnosticDecl::new("W0204", "log1p of -1 gave -Inf"),
                DiagnosticDecl::new("W0205", "log1p below -1 gave NA"),
            ],
        ),
        unary(
            "expm1",
            "Float",
            include_str!("../help/expm1.md"),
            expm1,
            vec![],
        ),
        unary(
            "sin",
            "Float",
            include_str!("../help/sin.md"),
            sin,
            vec![DiagnosticDecl::new(
                "W0205",
                "sin of an infinite value gave NA",
            )],
        ),
        unary(
            "cos",
            "Float",
            include_str!("../help/cos.md"),
            cos,
            vec![DiagnosticDecl::new(
                "W0205",
                "cos of an infinite value gave NA",
            )],
        ),
        unary(
            "tan",
            "Float",
            include_str!("../help/tan.md"),
            tan,
            vec![DiagnosticDecl::new(
                "W0205",
                "tan of an infinite value gave NA",
            )],
        ),
        unary(
            "asin",
            "Float",
            include_str!("../help/asin.md"),
            asin,
            vec![DiagnosticDecl::new("W0205", "asin outside [-1, 1] gave NA")],
        ),
        unary(
            "acos",
            "Float",
            include_str!("../help/acos.md"),
            acos,
            vec![DiagnosticDecl::new("W0205", "acos outside [-1, 1] gave NA")],
        ),
        unary(
            "atan",
            "Float",
            include_str!("../help/atan.md"),
            atan,
            vec![],
        ),
        Function {
            spec: FunctionSpec {
                name: "atan2",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("y", "Float"),
                    ParamSpec::required("x", "Float"),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![],
                help: include_str!("../help/atan2.md"),
                accuracy: None,
            },
            body: atan2,
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
