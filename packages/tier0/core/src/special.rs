//! The special functions of `core` (release 0.3.0, step 6; Language Reference, sections 3.7 and
//! 4.1): `factorial`, `choose`, `gamma`, `log_gamma` and `beta`, on the kernel of
//! `zeff_numeric::special`, with the warnings of section 4.1 where a value has no finite answer
//! (W0205 for `NA`, W0204 for an infinity). Each applies to a column row by row through the lift.

use crate::maths::{Number, as_float, float, gave_inf, gave_na, number};
use crate::runtime::{Args, CallContext, Fail, Function};
use crate::value::Value;
use zeff_numeric::special;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The Int in an argument, or `None` for `NA`; the checker admits no Float here.
fn int(value: Option<&Value>) -> Option<i64> {
    match number(value) {
        Number::Int(i) => Some(i),
        Number::Float(x) if x.fract() == 0.0 && x.abs() < 9.0e15 => Some(x as i64),
        _ => None,
    }
}

fn factorial(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(n) = int(args.get(0)) else {
        return Ok(Value::Na);
    };
    if n < 0 {
        gave_na(
            ctx,
            "factorial of a negative number",
            "n! is defined for whole numbers from 0; `gamma(x + 1)` extends it to other numbers",
        );
        return Ok(Value::Na);
    }
    if n > 170 {
        gave_inf(
            ctx,
            "factorial above 170",
            false,
            "171! exceeds the largest Float; `log_gamma(n + 1)` gives the logarithm, which stays finite",
        );
        return Ok(Value::Float(f64::INFINITY));
    }
    Ok(float(special::factorial(n)))
}

fn choose(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(n), Some(k)) = (int(args.get(0)), int(args.get(1))) else {
        return Ok(Value::Na);
    };
    if n < 0 {
        gave_na(
            ctx,
            "choose with a negative n",
            "the number of ways to choose k of n needs a whole number n of 0 or more",
        );
        return Ok(Value::Na);
    }
    let value = special::choose(n, k);
    if value.is_infinite() {
        gave_inf(
            ctx,
            "choose beyond the largest Float",
            false,
            "`log_gamma(n + 1) - log_gamma(k + 1) - log_gamma(n - k + 1)` gives the logarithm, which stays finite",
        );
    }
    Ok(float(value))
}

/// Whether `x` is zero or a negative whole number, where Γ has a pole.
fn at_pole(x: f64) -> bool {
    x <= 0.0 && x.fract() == 0.0
}

fn gamma(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(x) = as_float(number(args.get(0))) else {
        return Ok(Value::Na);
    };
    if at_pole(x) {
        gave_na(
            ctx,
            "gamma at a pole",
            "Γ has poles at zero and the negative whole numbers; between them it is finite",
        );
        return Ok(Value::Na);
    }
    let value = special::gamma(x);
    if value.is_infinite() {
        gave_inf(
            ctx,
            "gamma above 171.6",
            false,
            "Γ(x) exceeds the largest Float; `log_gamma(x)` gives the logarithm, which stays finite",
        );
    }
    Ok(float(value))
}

fn log_gamma(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(x) = as_float(number(args.get(0))) else {
        return Ok(Value::Na);
    };
    if at_pole(x) {
        gave_inf(
            ctx,
            "log_gamma at a pole",
            false,
            "Γ has poles at zero and the negative whole numbers, so its logarithm is infinite there",
        );
        return Ok(Value::Float(f64::INFINITY));
    }
    Ok(float(special::log_gamma(x)))
}

fn beta(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(a), Some(b)) = (as_float(number(args.get(0))), as_float(number(args.get(1)))) else {
        return Ok(Value::Na);
    };
    if a <= 0.0 || b <= 0.0 {
        gave_na(
            ctx,
            "beta with an argument that is not positive",
            "B(a, b) is defined for positive a and b",
        );
        return Ok(Value::Na);
    }
    let value = special::beta(a, b);
    if value.is_infinite() {
        gave_inf(
            ctx,
            "beta of tiny arguments",
            false,
            "B(a, b) exceeds the largest Float; `log_gamma(a) + log_gamma(b) - log_gamma(a + b)` gives the logarithm",
        );
    }
    Ok(float(value))
}

fn na_warning(when: &'static str) -> DiagnosticDecl {
    DiagnosticDecl::new("W0205", when)
}

fn inf_warning(when: &'static str) -> DiagnosticDecl {
    DiagnosticDecl::new("W0204", when)
}

fn function(
    name: &'static str,
    params: Vec<ParamSpec>,
    help: &'static str,
    body: crate::runtime::Body,
    diagnostics: Vec<DiagnosticDecl>,
) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "core",
            method: false,
            params,
            rest: None,
            returns: "Float",
            diagnostics,
            help,
            accuracy: None,
        },
        body,
    }
}

/// The special functions of `core` in this release.
pub fn functions() -> Vec<Function> {
    vec![
        function(
            "factorial",
            vec![ParamSpec::required("n", "Int")],
            include_str!("../help/factorial.md"),
            factorial,
            vec![
                na_warning("a negative n gave NA"),
                inf_warning("n above 170 gave Inf"),
            ],
        ),
        function(
            "choose",
            vec![
                ParamSpec::required("n", "Int"),
                ParamSpec::required("k", "Int"),
            ],
            include_str!("../help/choose.md"),
            choose,
            vec![
                na_warning("a negative n gave NA"),
                inf_warning("a result beyond the largest Float gave Inf"),
            ],
        ),
        function(
            "gamma",
            vec![ParamSpec::required("x", "Float")],
            include_str!("../help/gamma.md"),
            gamma,
            vec![
                na_warning("x at a pole (zero or a negative whole number) gave NA"),
                inf_warning("x above 171.6 gave Inf"),
            ],
        ),
        function(
            "log_gamma",
            vec![ParamSpec::required("x", "Float")],
            include_str!("../help/log_gamma.md"),
            log_gamma,
            vec![inf_warning(
                "x at a pole (zero or a negative whole number) gave Inf",
            )],
        ),
        function(
            "beta",
            vec![
                ParamSpec::required("a", "Float"),
                ParamSpec::required("b", "Float"),
            ],
            include_str!("../help/beta.md"),
            beta,
            vec![
                na_warning("an argument that is not positive gave NA"),
                inf_warning("a result beyond the largest Float gave Inf"),
            ],
        ),
    ]
}
