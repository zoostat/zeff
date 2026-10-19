//! Package `dist` (tier 1): probability distributions as values (Language Reference, sections 3.7
//! "Namespaces" and 4; Engine and Project Specification, section 8.2). `dist.normal(...)` and its
//! kin build a [`Distribution`] value with every default filled in, checking each parameter's
//! domain (Z0413); the methods `pdf`, `cdf`, `quantile` and `draw` arrive with the families.

pub mod families;

use zeff_columns::Column;
use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::{Distribution, Value};
use zeff_diagnostics::Diagnostic;
use zeff_registry::{AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec};

/// A parameter of a family: its name, its default, and the rule its value must satisfy.
struct Param {
    name: &'static str,
    default: f64,
    rule: Rule,
}

#[derive(Clone, Copy)]
enum Rule {
    Any,
    Positive,
}

impl Rule {
    fn holds(self, value: f64) -> bool {
        match self {
            Rule::Any => value.is_finite(),
            Rule::Positive => value.is_finite() && value > 0.0,
        }
    }

    fn words(self) -> &'static str {
        match self {
            Rule::Any => "a finite number",
            Rule::Positive => "a positive number",
        }
    }
}

/// Read a family's parameters from the arguments, in order, each checked against its rule;
/// `NA` or a value outside the domain is Z0413, pointing at the argument.
fn family(
    ctx: &CallContext<'_>,
    name: &'static str,
    params: &[Param],
    args: &Args,
) -> Result<Value, Fail> {
    let mut filled = Vec::with_capacity(params.len());
    for (i, param) in params.iter().enumerate() {
        let value = match args.get(i) {
            None => param.default,
            Some(Value::Float(x)) => *x,
            Some(Value::Int(n)) => *n as f64,
            Some(Value::Na) => f64::NAN,
            Some(other) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z9001",
                        format!(
                            "internal error: this is a bug in Zeff, not in your script (`{}` of `dist.{name}` reached the body as {})",
                            param.name,
                            other.type_name()
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "the call that failed"),
                ));
            }
        };
        if !param.rule.holds(value) {
            let shown = match args.get(i) {
                Some(Value::Na) => "NA".to_string(),
                _ => zeff_core::display::format_float(value),
            };
            return Err(fail(
                Diagnostic::error(
                    "Z0413",
                    format!(
                        "`{}` of `dist.{name}` must be {}, but this is {shown}",
                        param.name,
                        param.rule.words()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(i), "outside the parameter's domain")
                .help(format!(
                    "`zeff help dist.{name}` gives the domain of every parameter"
                )),
            ));
        }
        filled.push((param.name, value));
    }
    Ok(Value::Distribution(Distribution::Family {
        name,
        params: filled,
    }))
}

/// A rule between parameters, checked after each is in its domain: the parameter it points at,
/// and the condition with its words.
struct Joint {
    at: &'static str,
    holds: fn(&[(&'static str, f64)]) -> bool,
    words: &'static str,
}

fn family_checked(
    ctx: &CallContext<'_>,
    name: &'static str,
    params: &[Param],
    joint: Option<&Joint>,
    args: &Args,
) -> Result<Value, Fail> {
    let value = family(ctx, name, params, args)?;
    if let (Some(joint), Value::Distribution(Distribution::Family { params: filled, .. })) =
        (joint, &value)
        && !(joint.holds)(filled)
    {
        let at = params.iter().position(|p| p.name == joint.at).unwrap_or(0);
        return Err(fail(
            Diagnostic::error(
                "Z0413",
                format!("`{}` of `dist.{name}` must {}", joint.at, joint.words),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at), "outside the parameter's domain")
            .help(format!(
                "`zeff help dist.{name}` gives the domain of every parameter"
            )),
        ));
    }
    Ok(value)
}

const NORMAL: [Param; 2] = [
    Param {
        name: "mean",
        default: 0.0,
        rule: Rule::Any,
    },
    Param {
        name: "sd",
        default: 1.0,
        rule: Rule::Positive,
    },
];

/// `dist.normal(mean: Float = 0, sd: Float = 1)`.
pub fn normal(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "normal", &NORMAL, &args)
}

const LOGNORMAL: [Param; 2] = [
    Param {
        name: "meanlog",
        default: 0.0,
        rule: Rule::Any,
    },
    Param {
        name: "sdlog",
        default: 1.0,
        rule: Rule::Positive,
    },
];

/// `dist.lognormal(meanlog: Float = 0, sdlog: Float = 1)`.
pub fn lognormal(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "lognormal", &LOGNORMAL, &args)
}

const UNIFORM: [Param; 2] = [
    Param {
        name: "min",
        default: 0.0,
        rule: Rule::Any,
    },
    Param {
        name: "max",
        default: 1.0,
        rule: Rule::Any,
    },
];

const UNIFORM_ORDER: Joint = Joint {
    at: "max",
    holds: |p| p[0].1 < p[1].1,
    words: "exceed `min`",
};

/// `dist.uniform(min: Float = 0, max: Float = 1)`.
pub fn uniform(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family_checked(ctx, "uniform", &UNIFORM, Some(&UNIFORM_ORDER), &args)
}

const EXPONENTIAL: [Param; 1] = [Param {
    name: "rate",
    default: 1.0,
    rule: Rule::Positive,
}];

/// `dist.exponential(rate: Float = 1)`.
pub fn exponential(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "exponential", &EXPONENTIAL, &args)
}

// ----- the methods on a distribution -----

/// The distribution a method was called on.
fn receiver<'a>(ctx: &CallContext<'_>, args: &'a Args) -> Result<&'a Distribution, Fail> {
    match args.get(0) {
        Some(Value::Distribution(d)) => Ok(d),
        other => Err(fail(
            Diagnostic::error(
                "Z9001",
                format!(
                    "internal error: this is a bug in Zeff, not in your script (a distribution method reached its body with {} as the receiver)",
                    other.map_or("nothing", Value::type_name)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ))
    }
}

/// A Float argument, or `None` for `NA`.
fn float_arg(args: &Args, i: usize) -> Option<f64> {
    match args.get(i) {
        Some(Value::Float(x)) => Some(*x),
        Some(Value::Int(n)) => Some(*n as f64),
        _ => None,
    }
}

fn as_value(x: f64) -> Value {
    if x.is_nan() {
        Value::Na
    } else {
        Value::Float(x)
    }
}

/// `distribution.pdf(x)`.
pub fn pdf(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let d = receiver(ctx, &args)?;
    Ok(float_arg(&args, 1).map_or(Value::Na, |x| as_value(families::pdf(d, x))))
}

/// `distribution.cdf(x, upper: Bool = false)`.
pub fn cdf(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let d = receiver(ctx, &args)?;
    let upper = matches!(args.get(2), Some(Value::Bool(true)));
    Ok(float_arg(&args, 1).map_or(Value::Na, |x| as_value(families::cdf(d, x).tail(upper))))
}

/// `distribution.quantile(p)`.
pub fn quantile(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let d = receiver(ctx, &args)?;
    let Some(p) = float_arg(&args, 1) else {
        return Ok(Value::Na);
    };
    if !(0.0..=1.0).contains(&p) {
        return Err(fail(
            Diagnostic::error(
                "Z0413",
                format!(
                    "`p` of `quantile` must be a probability between 0 and 1, but this is {}",
                    zeff_core::display::format_float(p)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "not a probability"),
        ));
    }
    Ok(as_value(families::quantile(d, p)))
}

/// `distribution.draw(n: Int, seed: Int)`: `n` draws by inverse CDF from the uniform stream of
/// this statement, or of `seed:` alone when it is given; an Int column for a discrete family
/// (the owner's decision of 19 October 2026 on question 3), a Float column otherwise.
pub fn draw(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let d = receiver(ctx, &args)?.clone();
    let n = match args.get(1) {
        Some(Value::Int(n)) if *n >= 0 => *n as usize,
        Some(Value::Int(n)) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`n` of `draw` must be 0 or more, but this is {n}"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "a negative count"),
            ));
        }
        _ => {
            return Err(fail(
                Diagnostic::error("Z0401", "`n` of `draw` must be an Int, but this is NA")
                    .in_file(ctx.script)
                    .primary(ctx.span_of(1), "not a count"),
            ));
        }
    };
    let seed = match args.get(2) {
        Some(Value::Int(s)) => Some(*s),
        _ => None,
    };
    let mut stream = ctx.stream(seed);
    if families::is_discrete(&d) {
        let values: Vec<Option<i64>> = (0..n)
            .map(|_| {
                let x = families::draw(&d, stream.uniform());
                if x.is_finite() { Some(x as i64) } else { None }
            })
            .collect();
        return Ok(Value::Column(Column::ints(values)));
    }
    let (column, _) = Column::floats((0..n).map(|_| Some(families::draw(&d, stream.uniform()))));
    Ok(Value::Column(column))
}

fn z0413() -> DiagnosticDecl {
    DiagnosticDecl::new("Z0413", "a parameter outside its domain, or NA")
}

fn accuracy(reference: &str) -> Option<AccuracyRecord> {
    Some(AccuracyRecord {
        reference: reference.to_string(),
        last_run: None,
    })
}

/// A family's constructor.
fn constructor(
    name: &'static str,
    params: Vec<ParamSpec>,
    help: &'static str,
    reference: &str,
    body: zeff_core::runtime::Body,
) -> Function {
    Function {
        spec: FunctionSpec {
            name,
            package: "dist",
            method: false,
            params,
            rest: None,
            returns: "Distribution",
            diagnostics: vec![z0413()],
            help,
            accuracy: accuracy(reference),
        },
        body,
    }
}

/// A method on a distribution.
fn method(
    name: &'static str,
    params: Vec<ParamSpec>,
    returns: &'static str,
    help: &'static str,
    diagnostics: Vec<DiagnosticDecl>,
    body: zeff_core::runtime::Body,
) -> Function {
    let mut all = vec![ParamSpec::required("distribution", "Distribution")];
    all.extend(params);
    Function {
        spec: FunctionSpec {
            name,
            package: "dist",
            method: true,
            params: all,
            rest: None,
            returns,
            diagnostics,
            help,
            accuracy: accuracy(
                "R 4.6.0 (the d, p, q and r functions of each family) and the exact decimal reference of the kernel (tests/reference/special.py)",
            ),
        },
        body,
    }
}

/// The functions of the package, for the engine's registry.
pub fn functions() -> Vec<Function> {
    vec![
        constructor(
            "dist.normal",
            vec![
                ParamSpec::option_with_default("mean", "Float", "0"),
                ParamSpec::option_with_default("sd", "Float", "1"),
            ],
            include_str!("../help/dist.normal.md"),
            "R 4.6.0 dnorm, pnorm, qnorm and an exact decimal reference (tests/reference/special.py)",
            normal,
        ),
        constructor(
            "dist.lognormal",
            vec![
                ParamSpec::option_with_default("meanlog", "Float", "0"),
                ParamSpec::option_with_default("sdlog", "Float", "1"),
            ],
            include_str!("../help/dist.lognormal.md"),
            "R 4.6.0 dlnorm, plnorm, qlnorm",
            lognormal,
        ),
        constructor(
            "dist.uniform",
            vec![
                ParamSpec::option_with_default("min", "Float", "0"),
                ParamSpec::option_with_default("max", "Float", "1"),
            ],
            include_str!("../help/dist.uniform.md"),
            "R 4.6.0 dunif, punif, qunif",
            uniform,
        ),
        constructor(
            "dist.exponential",
            vec![ParamSpec::option_with_default("rate", "Float", "1")],
            include_str!("../help/dist.exponential.md"),
            "R 4.6.0 dexp, pexp, qexp",
            exponential,
        ),
        method(
            "pdf",
            vec![ParamSpec::required("x", "Float")],
            "Float",
            include_str!("../help/pdf.md"),
            vec![],
            pdf,
        ),
        method(
            "cdf",
            vec![
                ParamSpec::required("x", "Float"),
                ParamSpec::option_with_default("upper", "Bool", "false"),
            ],
            "Float",
            include_str!("../help/cdf.md"),
            vec![],
            cdf,
        ),
        method(
            "quantile",
            vec![ParamSpec::required("p", "Float")],
            "Float",
            include_str!("../help/quantile.md"),
            vec![DiagnosticDecl::new("Z0413", "p outside [0, 1]")],
            quantile,
        ),
        method(
            "draw",
            vec![
                ParamSpec::required("n", "Int"),
                ParamSpec::option("seed", "Int"),
            ],
            "Column<Float>",
            include_str!("../help/draw.md"),
            vec![
                DiagnosticDecl::new("Z0401", "a negative n"),
                DiagnosticDecl::note("the seed chosen for the run, when none was set"),
            ],
            draw,
        ),
    ]
}
