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
    /// A whole number of 0 or more.
    Count,
    /// In [0, 1].
    Probability,
    /// In (0, 1].
    ProbabilityNotZero,
}

impl Rule {
    fn holds(self, value: f64) -> bool {
        match self {
            Rule::Any => value.is_finite(),
            Rule::Positive => value.is_finite() && value > 0.0,
            Rule::Count => value.is_finite() && value >= 0.0 && value.fract() == 0.0,
            Rule::Probability => (0.0..=1.0).contains(&value),
            Rule::ProbabilityNotZero => value > 0.0 && value <= 1.0,
        }
    }

    fn words(self) -> &'static str {
        match self {
            Rule::Any => "a finite number",
            Rule::Positive => "a positive number",
            Rule::Count => "a whole number of 0 or more",
            Rule::Probability => "a probability between 0 and 1",
            Rule::ProbabilityNotZero => "a probability above 0 and at most 1",
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
    let positions: Vec<usize> = (0..params.len()).collect();
    family_with_spans(ctx, name, params, args, &positions)
}

/// `family` with each parameter's argument position given, for a call whose arguments were
/// reordered.
fn family_with_spans(
    ctx: &CallContext<'_>,
    name: &'static str,
    params: &[Param],
    args: &Args,
    positions: &[usize],
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
                .primary(ctx.span_of(positions[i]), "outside the parameter's domain")
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

const T: [Param; 1] = [Param {
    name: "df",
    default: f64::NAN,
    rule: Rule::Positive,
}];

/// `dist.t(df: Float)`.
pub fn t(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "t", &T, &args)
}

/// `dist.chi2(df: Float)`.
pub fn chi2(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "chi2", &T, &args)
}

const F: [Param; 2] = [
    Param {
        name: "df1",
        default: f64::NAN,
        rule: Rule::Positive,
    },
    Param {
        name: "df2",
        default: f64::NAN,
        rule: Rule::Positive,
    },
];

/// `dist.f(df1: Float, df2: Float)`.
pub fn f(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "f", &F, &args)
}

const GAMMA: [Param; 2] = [
    Param {
        name: "shape",
        default: f64::NAN,
        rule: Rule::Positive,
    },
    Param {
        name: "rate",
        default: 1.0,
        rule: Rule::Positive,
    },
];

/// `dist.gamma(shape: Float, rate: Float = 1)`.
pub fn gamma(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "gamma", &GAMMA, &args)
}

const BETA: [Param; 2] = [
    Param {
        name: "a",
        default: f64::NAN,
        rule: Rule::Positive,
    },
    Param {
        name: "b",
        default: f64::NAN,
        rule: Rule::Positive,
    },
];

/// `dist.beta(a: Float, b: Float)`.
pub fn beta(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "beta", &BETA, &args)
}

const BINOMIAL: [Param; 2] = [
    Param {
        name: "n",
        default: f64::NAN,
        rule: Rule::Count,
    },
    Param {
        name: "p",
        default: f64::NAN,
        rule: Rule::Probability,
    },
];

/// `dist.binomial(n: Int, p: Float)`.
pub fn binomial(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "binomial", &BINOMIAL, &args)
}

const POISSON: [Param; 1] = [Param {
    name: "lambda",
    default: f64::NAN,
    rule: Rule::Positive,
}];

/// `dist.poisson(lambda: Float)`.
pub fn poisson(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    family(ctx, "poisson", &POISSON, &args)
}

const NEGBINOMIAL_P: [Param; 2] = [
    Param {
        name: "size",
        default: f64::NAN,
        rule: Rule::Positive,
    },
    Param {
        name: "p",
        default: f64::NAN,
        rule: Rule::ProbabilityNotZero,
    },
];

const NEGBINOMIAL_MU: [Param; 2] = [
    Param {
        name: "mu",
        default: f64::NAN,
        rule: Rule::Positive,
    },
    Param {
        name: "size",
        default: f64::NAN,
        rule: Rule::Positive,
    },
];

/// `dist.negbinomial(size: Float, p: Float)` or `dist.negbinomial(mu: Float, size: Float)`: one
/// of `p:` and `mu:`, never both (Z0410) nor neither (Z0406).
pub fn negbinomial(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    // The registered parameters are size, p, mu in that order.
    let has_p = args.get(1).is_some();
    let has_mu = args.get(2).is_some();
    match (has_p, has_mu) {
        (true, true) => Err(fail(
            Diagnostic::error(
                "Z0410",
                "`dist.negbinomial` takes `p:` or `mu:`, not both",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "a second parameterisation")
            .help("`size:` with `p:` counts failures before the size-th success; `mu:` with `size:` is the mean parameterisation (NB2)"),
        )),
        (false, false) => Err(fail(
            Diagnostic::error("Z0406", "`dist.negbinomial` needs `p:` or `mu:`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "neither is given")
                .help("`dist.negbinomial(size: 3, p: 0.4)` or `dist.negbinomial(mu: 4.5, size: 3)`"),
        )),
        (true, false) => family(ctx, "negbinomial", &NEGBINOMIAL_P, &args),
        (false, true) => {
            // Reorder the arguments to (mu, size) for the table.
            let reordered = Args {
                values: vec![args.get(2).cloned(), args.get(0).cloned()],
                rest: Vec::new(),
                definitions: Vec::new(),
            };
            family_with_spans(ctx, "negbinomial", &NEGBINOMIAL_MU, &reordered, &[2, 0])
        }
    }
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
        constructor(
            "dist.t",
            vec![ParamSpec::required("df", "Float")],
            include_str!("../help/dist.t.md"),
            "R 4.6.0 dt, pt, qt",
            t,
        ),
        constructor(
            "dist.chi2",
            vec![ParamSpec::required("df", "Float")],
            include_str!("../help/dist.chi2.md"),
            "R 4.6.0 dchisq, pchisq, qchisq",
            chi2,
        ),
        constructor(
            "dist.f",
            vec![
                ParamSpec::required("df1", "Float"),
                ParamSpec::required("df2", "Float"),
            ],
            include_str!("../help/dist.f.md"),
            "R 4.6.0 df, pf, qf",
            f,
        ),
        constructor(
            "dist.gamma",
            vec![
                ParamSpec::required("shape", "Float"),
                ParamSpec::option_with_default("rate", "Float", "1"),
            ],
            include_str!("../help/dist.gamma.md"),
            "R 4.6.0 dgamma, pgamma, qgamma and the exact decimal reference of the kernel",
            gamma,
        ),
        constructor(
            "dist.beta",
            vec![
                ParamSpec::required("a", "Float"),
                ParamSpec::required("b", "Float"),
            ],
            include_str!("../help/dist.beta.md"),
            "R 4.6.0 dbeta, pbeta, qbeta and the exact decimal reference of the kernel",
            beta,
        ),
        constructor(
            "dist.binomial",
            vec![
                ParamSpec::required("n", "Int"),
                ParamSpec::required("p", "Float"),
            ],
            include_str!("../help/dist.binomial.md"),
            "R 4.6.0 dbinom, pbinom, qbinom",
            binomial,
        ),
        constructor(
            "dist.poisson",
            vec![ParamSpec::required("lambda", "Float")],
            include_str!("../help/dist.poisson.md"),
            "R 4.6.0 dpois, ppois, qpois",
            poisson,
        ),
        Function {
            spec: FunctionSpec {
                name: "dist.negbinomial",
                package: "dist",
                method: false,
                params: vec![
                    ParamSpec::required("size", "Float"),
                    ParamSpec::option("p", "Float"),
                    ParamSpec::option("mu", "Float"),
                ],
                rest: None,
                returns: "Distribution",
                diagnostics: vec![
                    z0413(),
                    DiagnosticDecl::new("Z0410", "both `p:` and `mu:` given"),
                    DiagnosticDecl::new("Z0406", "neither `p:` nor `mu:` given"),
                ],
                help: include_str!("../help/dist.negbinomial.md"),
                accuracy: accuracy("R 4.6.0 dnbinom, pnbinom, qnbinom"),
            },
            body: negbinomial,
        },
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
