//! The options a model shares with `reg` (Language Reference, section 6: `se:`, `cluster:`,
//! `weights:`, `weight_type:` and `level:`), read and checked in one place for every model
//! package, since no model package may call another (section 8.1, rule 5): the kind of standard
//! error and the combinations refused, the variance from the variance layer with its refusals
//! and its self-check, the lines the summary prints for the errors, the observations and the
//! weights, and the notes and statistical warnings that clusters and weights raise (W1002,
//! W1003). Moved here from `regress` at release 0.9.0, step 7a, when `glm` came to share them.

use crate::absorb::EffectGroups;
use crate::design::Design;
use crate::summary::{n_note, rows};
use crate::variance::{Clusters, Kind, VarianceError, effective_sample_size};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::Matrix;
use zeff_numeric::order::WeightKind;

/// The condition number above which the design is warned about (W1001, section 11.1).
pub const CONDITION_WARNING: f64 = 1e8;
/// Fewer clusters than this is warned about (W1002).
pub const FEW_CLUSTERS: usize = 30;
/// Weights whose effective sample size is below this share of the rows are warned about (W1003).
pub const EXTREME_WEIGHTS_SHARE: f64 = 0.25;

/// A variance from the variance layer, with what it could not compute turned into the refusal
/// a script reads (Z0802: an observation with leverage one under `hc2` or `hc3`, one cluster, no
/// residual degrees of freedom), and the self-check of section 11.1: a symmetric matrix with no
/// negative variance. `cluster` is the cluster variable and `cluster_at` its argument's place.
pub fn checked_variance(
    ctx: &mut CallContext<'_>,
    computed: Result<(Matrix, usize), VarianceError>,
    kind: Kind,
    cluster: Option<&str>,
    cluster_at: usize,
) -> Result<(Matrix, usize), Fail> {
    let (vcov, df) = match computed {
        Ok(pair) => pair,
        Err(VarianceError::Asymmetric) => {
            return Err(internal(ctx, "an asymmetric variance matrix"));
        }
        Err(VarianceError::LeverageOne(rows_at_one)) => {
            let word = match kind {
                Kind::Hc3 => "hc3",
                _ => "hc2",
            };
            return Err(fail(
                    Diagnostic::error(
                        "Z0802",
                        format!(
                            "`se: {word}` needs every observation's leverage below one, but {} leverage one: each is alone in a cell of the design",
                            if rows_at_one == 1 {
                                "1 row has".to_string()
                            } else {
                                format!("{} rows have", count(rows_at_one as u64))
                            }
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "an observation that fits itself exactly")
                    .help("`se: robust` (HC1) keeps these rows; or drop the regressor that singles each out, such as a category level with one observation"),
                ));
        }
        Err(VarianceError::OneCluster) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0802",
                    format!(
                        "clustered errors need at least two clusters, and `{}` has one",
                        cluster.unwrap_or("")
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(cluster_at), "one cluster")
                .help("cluster by a variable with several values, or leave `cluster:` out"),
            ));
        }
    };
    if df == 0 {
        return Err(fail(
            Diagnostic::error(
                "Z0802",
                "no residual degrees of freedom: as many regressors as observations",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "nothing left to estimate the error from")
            .help("a model needs more observations than regressors; drop terms or use more data"),
        ));
    }
    // Self-check (section 11.1): a symmetric variance matrix with no negative variance.
    let k = vcov.rows();
    for i in 0..k {
        if vcov.get(i, i).is_nan() || vcov.get(i, i) < 0.0 {
            return Err(internal(ctx, "a negative variance"));
        }
        for j in 0..i {
            let (a, b) = (vcov.get(i, j), vcov.get(j, i));
            if (a - b).abs() > 1e-10 * a.abs().max(b.abs()).max(f64::MIN_POSITIVE) {
                return Err(internal(ctx, "an asymmetric variance matrix"));
            }
        }
    }
    Ok((vcov, df))
}

/// The note and the warnings that clusters and weights raise for any model: clusters of one
/// observation (a note), fewer than [`FEW_CLUSTERS`] clusters (W1002), and weights whose
/// effective sample size is a small share of the rows (W1003). Each is printed; the notes and
/// the warnings are kept for the fit.
pub fn cluster_and_weight_warnings(
    ctx: &mut CallContext<'_>,
    notes: &mut Vec<String>,
    warnings: &mut Vec<String>,
    clusters: Option<&Clusters>,
    design: &Design,
) {
    if let Some(clusters) = clusters {
        if clusters.singletons > 0 {
            let text = format!(
                "{} of the {} clusters hold one observation",
                count(clusters.singletons as u64),
                count(clusters.count as u64)
            );
            notes.push(text.clone());
            ctx.message(Diagnostic::plain_note(text));
        }
        if clusters.count < FEW_CLUSTERS {
            let text = format!(
                "few clusters: {} clusters give clustered errors little to rest on",
                count(clusters.count as u64)
            );
            warnings.push(format!("warning[W1002]: {text}"));
            ctx.message(
                Diagnostic::warning("W1002", text)
                    .help("the wild cluster bootstrap (`se: bootstrap`, release 1.1) is the usual remedy below 30 clusters"),
            );
        }
    }
    if let Some(w) = &design.weights {
        let effective = effective_sample_size(w);
        let n = design.n() as f64;
        if n > 0.0 && effective < EXTREME_WEIGHTS_SHARE * n {
            let text = format!(
                "extreme weights: the effective sample size is {} of {} rows",
                count(effective.round() as u64),
                count(design.n() as u64)
            );
            warnings.push(format!("warning[W1003]: {text}"));
            ctx.message(
                Diagnostic::warning("W1003", text)
                    .help("a few rows carry most of the weight; check them, or trim the weights"),
            );
        }
    }
}

/// How the summary's line names the standard errors.
pub fn se_line(kind: Kind, clusters: Option<&Clusters>, cluster: Option<&str>) -> String {
    match kind {
        Kind::Classical => "classical".to_string(),
        Kind::Hc1 => "robust (HC1)".to_string(),
        Kind::Hc2 => "hc2".to_string(),
        Kind::Hc3 => "hc3".to_string(),
        Kind::Cluster => {
            let c = clusters.expect("clusters");
            format!(
                "clustered by {} ({} clusters, CR1)",
                cluster.unwrap_or(""),
                count(c.count as u64)
            )
        }
    }
}

/// The observation count the summary prints and what it says in brackets beside it: the rows
/// used, or the sum of the frequency weights and that it is one; the rows dropped for `NA` and
/// left out for a zero weight; and, with fixed effects (`singletons` given), the singletons.
pub fn observations(design: &Design, singletons: Option<usize>) -> (usize, Option<String>) {
    let n_obs = match design.weight_kind {
        Some(WeightKind::Frequency) => design.weight_total().round() as usize,
        _ => design.n(),
    };
    let mut note = n_note(design);
    if design.weight_kind == Some(WeightKind::Frequency) {
        let sum_text = format!("the sum of the frequency weights over {}", rows(design.n()));
        note = Some(match note {
            Some(rest) => format!("{sum_text}; {rest}"),
            None => sum_text,
        });
    }
    if let Some(singletons) = singletons {
        let singletons = match singletons {
            0 => "no singletons".to_string(),
            1 => "1 row dropped as a singleton".to_string(),
            n => format!("{} dropped as singletons", rows(n)),
        };
        note = Some(match note {
            Some(rest) => format!("{rest}; {singletons}"),
            None => singletons,
        });
    }
    (n_obs, note)
}

/// The summary's line for the weights, when the model is weighted.
pub fn weights_line(weights: Option<&str>, design: &Design) -> Vec<String> {
    match (weights, design.weight_kind) {
        (Some(name), Some(kind)) => vec![format!(
            "weights: {name} ({})",
            match kind {
                WeightKind::Frequency => "frequency",
                WeightKind::Analytic => "analytic",
                WeightKind::Sampling => "sampling",
            }
        )],
        _ => Vec::new(),
    }
}

/// The positions of the options a model shares with `reg`, among a function's arguments.
pub struct At {
    pub se: usize,
    pub cluster: usize,
    pub weights: usize,
    pub weight_type: usize,
    pub level: usize,
}

/// What `se:`, `cluster:`, `weights:`, `weight_type:` and `level:` ask for, once checked.
pub struct Choices {
    pub cluster: Option<String>,
    pub weights: Option<String>,
    pub weight_kind: WeightKind,
    pub kind: Kind,
    pub level: f64,
}

/// The options a model shares with `reg`, read and checked (section 6; the rulings of release
/// 0.4.0): the kind of standard error from `se:`, `cluster:` and the kind of the weights, with
/// the combinations that exclude each other refused, and the confidence level. `effects` is
/// the number of fixed effects in `fe:`: with more than one, `hc2` and `hc3` are refused.
pub fn choices(
    ctx: &mut CallContext<'_>,
    args: &Args,
    at: &At,
    effects: usize,
) -> Result<Choices, Fail> {
    let cluster = match args.get(at.cluster) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weights = match args.get(at.weights) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weight_kind = match args.get(at.weight_type) {
        Some(Value::Word(word)) => match word.as_str() {
            "frequency" => WeightKind::Frequency,
            "analytic" => WeightKind::Analytic,
            _ => WeightKind::Sampling,
        },
        _ => WeightKind::Sampling,
    };
    let sampling = weights.is_some() && weight_kind == WeightKind::Sampling;
    let se_word = match args.get(at.se) {
        Some(Value::Word(word)) => Some(word.clone()),
        _ => None,
    };
    if let (Some(word @ ("hc2" | "hc3")), true) = (se_word.as_deref(), effects > 1) {
        return Err(fail(
            Diagnostic::error(
                "Z0410",
                format!("`se: {word}` and several fixed effects exclude each other: with more than one absorbed effect an observation's leverage in the full design is not available"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at.se), "needs every observation's leverage")
            .help(format!("`se: robust` (HC1) or `cluster:` with several effects; with one effect in `fe:`, `se: {word}` is available; leverage under several effects arrives with release 1.1")),
        ));
    }
    let kind = match (se_word.as_deref(), cluster.is_some()) {
        (Some(word @ ("hc2" | "hc3")), true) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0410",
                    format!("`se: {word}` and `cluster:` exclude each other"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(at.se), "two kinds of standard error")
                .help("clustered errors are CR1; leave `se:` out with `cluster:`"),
            ));
        }
        (_, true) => Kind::Cluster,
        (None, false) => {
            if sampling {
                Kind::Hc1
            } else {
                Kind::Classical
            }
        }
        (Some("classical"), false) => {
            if sampling {
                return Err(fail(
                    Diagnostic::error(
                        "Z0410",
                        "`se: classical` with sampling weights: classical errors treat the weights as precision weights",
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(at.se), "not valid for probability weights")
                    .help("leave `se:` out, which gives `robust`, or write `weight_type: analytic` for precision weights"),
                ));
            }
            Kind::Classical
        }
        (Some("robust"), false) => Kind::Hc1,
        (Some("hc2"), false) => Kind::Hc2,
        (Some("hc3"), false) => Kind::Hc3,
        (Some(other), false) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0304",
                    format!("`se: {other}` is not available in this release"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(at.se), "arrives with release 1.1")
                .help("`classical`, `robust`, `hc2`, `hc3` and `cluster:` are available now"),
            ));
        }
    };
    let level = match args.get(at.level) {
        Some(Value::Float(level)) => *level,
        Some(Value::Int(level)) => *level as f64,
        _ => 0.95,
    };
    if !(level > 0.0 && level < 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`level:` is a probability strictly between 0 and 1, but this is {level}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at.level), "not a confidence level")
            .help("the usual levels are 0.9, 0.95 and 0.99"),
        ));
    }
    Ok(Choices {
        cluster,
        weights,
        weight_kind,
        kind,
        level,
    })
}

/// The clusters of the rows a design uses, from the cluster variable of the frame, by identity
/// of its values (the variable was given among `extra` when the design was built, so its `NA`
/// rows are already dropped).
pub fn clusters_of(
    ctx: &CallContext<'_>,
    frame: &Frame,
    name: &str,
    design: &Design,
) -> Result<Clusters, Fail> {
    let column = frame
        .table
        .column(name)
        .ok_or_else(|| internal(ctx, "the cluster variable vanished"))?;
    let groups = EffectGroups::over(
        name,
        std::slice::from_ref(&name.to_string()),
        &[column],
        &design.rows,
    );
    Ok(Clusters::from_groups(&groups.group_of_row, groups.levels()))
}

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
