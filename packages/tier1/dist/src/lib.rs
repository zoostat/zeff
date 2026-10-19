//! Package `dist` (tier 1): probability distributions as values (Language Reference, sections 3.7
//! "Namespaces" and 4; Engine and Project Specification, section 8.2). `dist.normal(...)` and its
//! kin build a [`Distribution`] value with every default filled in, checking each parameter's
//! domain (Z0413); the methods `pdf`, `cdf`, `quantile` and `draw` arrive with the families.

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

fn z0413() -> DiagnosticDecl {
    DiagnosticDecl::new("Z0413", "a parameter outside its domain, or NA")
}

/// The functions of the package, for the engine's registry.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "dist.normal",
            package: "dist",
            method: false,
            params: vec![
                ParamSpec::option_with_default("mean", "Float", "0"),
                ParamSpec::option_with_default("sd", "Float", "1"),
            ],
            rest: None,
            returns: "Distribution",
            diagnostics: vec![z0413()],
            help: include_str!("../help/dist.normal.md"),
            accuracy: Some(AccuracyRecord {
                reference: "R 4.6.0 dnorm, pnorm, qnorm and an exact decimal reference (tests/reference/special.py)".to_string(),
                last_run: None,
            }),
        },
        body: normal,
    }]
}
