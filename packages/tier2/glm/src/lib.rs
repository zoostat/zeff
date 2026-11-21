//! Package glm (tier 2): generalized linear models (Engine and Project Specification, sections
//! 8.2 and 14; open decision #112). At this release one function, `glm`, with the binomial
//! family and the logit and probit links, fitted by maximum likelihood through the shared
//! optimiser of the kernels and the likelihood trait of `results`; further families and links
//! are words added to `family:` and `link:` from release 1.7.

pub mod binary;

use zeff_core::runtime::Function;
use zeff_registry::{AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec};

/// The functions of `glm` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![Function {
        spec: FunctionSpec {
            name: "glm",
            package: "glm",
            method: true,
            params: vec![
                ParamSpec::required("data", "Frame"),
                ParamSpec::required("formula", "Formula"),
                ParamSpec::option("family", "binomial"),
                ParamSpec::option_with_default("link", "logit | probit", "logit"),
                ParamSpec::option("fe", "Terms"),
                ParamSpec::option("se", "classical | robust | hc2 | hc3"),
                ParamSpec::option("cluster", "Var"),
                ParamSpec::option("weights", "Var"),
                ParamSpec::option_with_default(
                    "weight_type",
                    "frequency | analytic | sampling",
                    "sampling",
                ),
                ParamSpec::option_with_default("level", "Float", "0.95"),
            ],
            rest: None,
            returns: "GlmFit",
            diagnostics: vec![
                DiagnosticDecl::note(
                    "a term written twice kept once; a collinear column, a level with no observations, a zero column or one too small or too large to scale dropped; rows with a zero weight left out; clusters of one observation",
                ),
                DiagnosticDecl::new(
                    "W1001",
                    "near-singular regressors, with the condition number",
                ),
                DiagnosticDecl::new("W1002", "fewer than 30 clusters"),
                DiagnosticDecl::new("W1003", "extreme weights, with the effective sample size"),
                DiagnosticDecl::new(
                    "W1008",
                    "fitted probabilities numerically 0 or 1, with their count",
                ),
                DiagnosticDecl::new("Z0406", "`family:` left out"),
                DiagnosticDecl::new("Z0224", "a negative weight"),
                DiagnosticDecl::new("Z0304", "`fe:`, which `glm` takes from release 1.7"),
                DiagnosticDecl::new(
                    "Z0410",
                    "`se: hc2` or `hc3` with `cluster:`; `se: classical` with sampling weights; `weight_type: analytic`",
                ),
                DiagnosticDecl::new(
                    "Z0401",
                    "an outcome that is not 0 and 1, or true and false, or that is a category; a level outside (0, 1); a weight that is not a number, or a frequency weight with a fraction",
                ),
                DiagnosticDecl::new(
                    "Z0101",
                    "a variable of the formula, `cluster:` or `weights:` is not in the frame",
                ),
                DiagnosticDecl::new(
                    "Z0801",
                    "no row remains to fit the model",
                ),
                DiagnosticDecl::new(
                    "Z0802",
                    "as many regressors as rows, or more; no regressor at all; one cluster; an observation with leverage one under `se: hc2` or `hc3`",
                ),
                DiagnosticDecl::new(
                    "Z0803",
                    "an outcome with one value in every row used, separation, a regressor the weighted design cannot tell from the others, a log-likelihood that is not finite, or a fit that has not converged in 100 iterations: maximum likelihood is not found",
                ),
            ],
            help: include_str!("../help/glm.md"),
            accuracy: Some(AccuracyRecord {
                reference: "R 4.6.0 glm(family = binomial) with the logit and probit links, iterated to its maximum (tests/reference/r/binary.R)".to_string(),
                last_run: None,
            }),
        },
        body: binary::glm,
    }]
}
