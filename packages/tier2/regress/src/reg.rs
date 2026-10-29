//! `reg` (Language Reference, sections 6, 7.1, 7.2 and 10.2; question 3 (a) of release 0.4.0):
//! the design of the formula on the frame, weighted when `weights:` is given, the least-squares
//! fit, the variance of the kind `se:` and `cluster:` ask for, the notes of what was changed, the
//! statistical warnings of section 11.1, and the fit value with its printed summary. A fitted
//! design is kept for the run, so that a second `reg` with another `se:` on the same frame and
//! formula reuses the factorisation (Engine and Project Specification, section 8.3, point 10).

use crate::ols::Ols;
use std::cell::RefCell;
use std::rc::Rc;
use zeff_core::display::cell;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::{Matrix, sum};
use zeff_numeric::order::WeightKind;
use zeff_results::design::Design;
use zeff_results::estimator::{Estimator, Fitted};
use zeff_results::summary::{self, Setting, n_note, rows};
use zeff_results::variance::{self, Clusters, Kind, VarianceError, effective_sample_size};

/// The condition number above which the design is warned about (W1001, section 11.1).
pub const CONDITION_WARNING: f64 = 1e8;
/// Fewer clusters than this is warned about (W1002).
pub const FEW_CLUSTERS: usize = 30;
/// Weights whose effective sample size is below this share of the rows are warned about (W1003).
pub const EXTREME_WEIGHTS_SHARE: f64 = 0.25;
/// An observation whose removal moves an estimate by more than this many standard errors drives
/// it (W1006): a DFBETAS beyond 1.
pub const DRIVING_DFBETAS: f64 = 1.0;

/// A design and its fit, kept for the run under the frame, the formula and the weighting that
/// made them, so that another `se:` or `cluster:` reuses the factorisation. The frame's table is
/// held, so that its address cannot be reused by another frame while the entry lives.
struct Kept {
    table: Rc<zeff_columns::Table>,
    key: String,
    design: Rc<Design>,
    fitted: Rc<Fitted>,
}

thread_local! {
    static KEPT: RefCell<Option<Kept>> = const { RefCell::new(None) };
}

/// `data.reg(formula, se:, cluster:, weights:, weight_type:, level:)`.
pub fn reg(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(ctx, "reg was called without a frame"));
    };
    let Some(Value::Formula(formula)) = args.get(1) else {
        return Err(internal(ctx, "reg was called without a formula"));
    };
    let frame = frame.clone();
    let formula = formula.clone();
    let cluster = match args.get(3) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weights = match args.get(4) {
        Some(Value::Var(name)) => Some(name.clone()),
        _ => None,
    };
    let weight_kind = match args.get(5) {
        Some(Value::Word(word)) => match word.as_str() {
            "frequency" => WeightKind::Frequency,
            "analytic" => WeightKind::Analytic,
            _ => WeightKind::Sampling,
        },
        _ => WeightKind::Sampling,
    };
    let sampling = weights.is_some() && weight_kind == WeightKind::Sampling;
    let se_word = match args.get(2) {
        Some(Value::Word(word)) => Some(word.clone()),
        _ => None,
    };
    let kind = match (se_word.as_deref(), cluster.is_some()) {
        (Some(word @ ("hc2" | "hc3")), true) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0410",
                    format!("`se: {word}` and `cluster:` exclude each other"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "two kinds of standard error")
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
                    .primary(ctx.span_of(2), "not valid for probability weights")
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
                .primary(ctx.span_of(2), "arrives with release 1.1")
                .help("`classical`, `robust`, `hc2`, `hc3` and `cluster:` are available now"),
            ));
        }
    };
    let level = match args.get(6) {
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
            .primary(ctx.span_of(6), "not a confidence level")
            .help("the usual levels are 0.9, 0.95 and 0.99"),
        ));
    }
    // The design and the fit, reused when this run already made them for this frame, formula
    // and weighting (the cluster's `NA` rows change the rows used, so it is part of the key).
    let key = format!(
        "{formula}|{}|{:?}|{}",
        weights.as_deref().unwrap_or(""),
        weights.as_ref().map(|_| weight_kind),
        cluster.as_deref().unwrap_or("")
    );
    let reused = KEPT.with(|kept| {
        kept.borrow().as_ref().and_then(|k| {
            (Rc::ptr_eq(&k.table, &frame.table) && k.key == key)
                .then(|| (Rc::clone(&k.design), Rc::clone(&k.fitted)))
        })
    });
    let (design, fitted) = match reused {
        Some(pair) => pair,
        None => {
            let extra: Vec<&str> = weights
                .iter()
                .chain(cluster.iter())
                .map(String::as_str)
                .collect();
            let mut design = Design::build(ctx, &frame, &formula, &extra)?;
            if let Some(name) = &weights {
                design = design.weighted(ctx, &frame, name, weight_kind)?;
            }
            let fitted = Ols.fit(ctx, &design)?;
            let (design, fitted) = (Rc::new(design), Rc::new(fitted));
            KEPT.with(|kept| {
                *kept.borrow_mut() = Some(Kept {
                    table: Rc::clone(&frame.table),
                    key,
                    design: Rc::clone(&design),
                    fitted: Rc::clone(&fitted),
                });
            });
            (design, fitted)
        }
    };
    for note in &design.notes {
        ctx.message(Diagnostic::plain_note(note.clone()));
    }
    // The notes printed under the fit are kept on it for the footer of a `report` table
    // (section 7.2); the design's own notes are there already.
    let mut notes: Vec<String> = Vec::new();
    if design.zero_weights > 0 {
        let text = format!(
            "{} with a zero weight left out of the estimate and the observation count",
            rows(design.zero_weights)
        );
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    // The columns dropped, each kind with its own note: a category level with no observations
    // among the rows used, a column that is zero in every row, a column whose values are too
    // small to be scaled, and the rest collinear with the columns before them.
    let names = |columns: &[usize]| -> Vec<String> {
        columns
            .iter()
            .map(|&c| format!("`{}`", design.names[c]))
            .collect()
    };
    let note = |ctx: &mut CallContext<'_>,
                notes: &mut Vec<String>,
                listed: Vec<String>,
                one: &str,
                several: &str| {
        let text = match listed.len() {
            0 => return,
            1 => format!("{} {one}", listed[0]),
            _ => format!("{} {several}", listed.join(", ")),
        };
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    };
    let (levels, zeros): (Vec<usize>, Vec<usize>) = fitted
        .zero
        .iter()
        .partition(|&&c| design.names[c].contains('='));
    note(
        ctx,
        &mut notes,
        names(&levels),
        "has no observations among the rows used and was dropped",
        "have no observations among the rows used and were dropped",
    );
    note(
        ctx,
        &mut notes,
        names(&zeros),
        "is zero in every row used and was dropped",
        "are zero in every row used and were dropped",
    );
    note(
        ctx,
        &mut notes,
        names(&fitted.vanishing),
        "has values too small to be scaled (below 1e-150 in size) and was dropped",
        "have values too small to be scaled (below 1e-150 in size) and were dropped",
    );
    note(
        ctx,
        &mut notes,
        names(&fitted.oversized),
        "has values too large to be scaled (above 1e150 in size) and was dropped",
        "have values too large to be scaled (above 1e150 in size) and were dropped",
    );
    let collinear: Vec<usize> = fitted
        .omitted
        .iter()
        .copied()
        .filter(|c| {
            !fitted.zero.contains(c)
                && !fitted.vanishing.contains(c)
                && !fitted.oversized.contains(c)
        })
        .collect();
    note(
        ctx,
        &mut notes,
        names(&collinear),
        "is collinear with the columns before it and was dropped",
        "are collinear with the columns before them and were dropped",
    );
    // The statistical warnings, printed here and kept on the fit for the footer of a `report`
    // table (section 11.1).
    let mut warnings: Vec<String> = Vec::new();
    if fitted.condition_number > CONDITION_WARNING {
        let text = format!(
            "near-singular regressors: the condition number of the scaled design is {:.3e}",
            fitted.condition_number
        );
        warnings.push(format!("warning[W1001]: {text}"));
        ctx.message(
            Diagnostic::warning("W1001", text)
                .help("two regressors are almost the same column; drop one, or combine them"),
        );
    }
    // The clusters of the rows used, from the cluster variable, by identity of its values.
    let clusters = match &cluster {
        Some(name) => {
            let column = frame
                .table
                .column(name)
                .ok_or_else(|| internal(ctx, "the cluster variable vanished"))?;
            let keys: Vec<String> = design
                .rows
                .iter()
                .map(|&row| cell(column.get(row).unwrap_or(zeff_columns::Value::Na)))
                .collect();
            Some(Clusters::from_keys(&keys))
        }
        None => None,
    };
    let bread = Ols.bread(&fitted);
    let (vcov, df) = match variance::variance(
        kind,
        &Ols,
        &design,
        &fitted,
        &bread,
        clusters.as_ref(),
    ) {
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
                        cluster.as_deref().unwrap_or("")
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(3), "one cluster")
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
    let k = fitted.kept.len();
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
    // The statistical warnings: few clusters (W1002), extreme weights (W1003), an observation
    // that drives a coefficient (W1006).
    if let Some(clusters) = &clusters {
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
    let driving = driving_observations(&design, &fitted, &bread);
    if !driving.is_empty() {
        let listed = driving
            .iter()
            .take(5)
            .map(|r| count(*r as u64))
            .collect::<Vec<_>>()
            .join(", ");
        let more = if driving.len() > 5 {
            format!(" and {} more", count((driving.len() - 5) as u64))
        } else {
            String::new()
        };
        let (observations, verb, which, plural) = if driving.len() == 1 {
            ("1 observation".to_string(), "drives", "it", "")
        } else {
            (
                format!("{} observations", count(driving.len() as u64)),
                "drive",
                "one",
                "s",
            )
        };
        let text = format!(
            "{observations} {verb} a coefficient: removing {which} moves an estimate by more than its standard error (row{plural} {listed}{more})"
        );
        warnings.push(format!("warning[W1006]: {text}"));
        ctx.message(
            Diagnostic::warning("W1006", text)
                .help("check the row's values; `influence()` lists leverage, Cook's distance and studentized residuals by row"),
        );
    }
    // The summary.
    let se_line = match kind {
        Kind::Classical => "classical".to_string(),
        Kind::Hc1 => "robust (HC1)".to_string(),
        Kind::Hc2 => "hc2".to_string(),
        Kind::Hc3 => "hc3".to_string(),
        Kind::Cluster => {
            let c = clusters.as_ref().expect("clusters");
            format!(
                "clustered by {} ({} clusters, CR1)",
                cluster.as_deref().unwrap_or(""),
                count(c.count as u64)
            )
        }
    };
    let n_obs = match design.weight_kind {
        Some(WeightKind::Frequency) => design.weight_total().round() as usize,
        _ => design.n(),
    };
    let mut note = n_note(&design);
    if design.weight_kind == Some(WeightKind::Frequency) {
        let sum_text = format!("the sum of the frequency weights over {}", rows(design.n()));
        note = Some(match note {
            Some(rest) => format!("{sum_text}; {rest}"),
            None => sum_text,
        });
    }
    let more_lines = match (&weights, design.weight_kind) {
        (Some(name), Some(kind)) => vec![format!(
            "weights: {name} ({})",
            match kind {
                WeightKind::Frequency => "frequency",
                WeightKind::Analytic => "analytic",
                WeightKind::Sampling => "sampling",
            }
        )],
        _ => Vec::new(),
    };
    let formula_text = formula.to_string();
    let setting = Setting {
        method: "reg",
        formula: &formula_text,
        se_line: &se_line,
        df,
        level,
        n: n_obs,
        n_note: note,
        more_lines,
        model: formula.clone(),
        levels: design.category_levels(),
        frame_rows: frame.table.rows(),
    };
    let mut fit = summary::linear_fit(&design, &fitted, &vcov, &setting);
    fit.warnings = warnings;
    fit.notes.extend(notes);
    // The residuals and fitted values of the rows, unweighted.
    if let Some(w) = &design.weights {
        for (i, root) in w.iter().map(|w| w.sqrt()).enumerate() {
            fit.residuals[i] /= root;
            fit.fitted[i] /= root;
        }
    }
    if fit.r_squared.is_none() {
        let outcome = formula
            .outcome
            .as_ref()
            .map_or(String::new(), |o| o.text.clone());
        let text = format!("`{outcome}` has no variation, so R² is undefined");
        fit.notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
    }
    if df > 0 && essentially_perfect(fit.sigma, &fit.fitted) {
        let text =
            "the fit is essentially perfect, so the standard errors and t values say nothing";
        fit.notes.push(text.to_string());
        ctx.message(Diagnostic::plain_note(text));
    }
    Ok(Value::Fit(Rc::new(fit)))
}

/// The rows (1-based, in the frame) whose removal moves some estimate by more than
/// `DRIVING_DFBETAS` standard errors: DFBETAS from the leave-one-out formulas, which need only
/// the row, the bread, its residual and its leverage.
fn driving_observations(design: &Design, fitted: &Fitted, bread: &Matrix) -> Vec<usize> {
    let k = fitted.kept.len();
    let n = design.n();
    if n <= k + 1 {
        return Vec::new();
    }
    let rss = fitted.residual_sum_of_squares;
    let df = (n - k) as f64;
    let mut driving = Vec::new();
    let mut xi = vec![0.0; k];
    for block in design.blocks() {
        for r in 0..block.x.rows() {
            let row = block.start + r;
            let e = fitted.residuals[row];
            let h = fitted.leverage[row];
            if h >= 1.0 {
                continue;
            }
            for (p, &c) in fitted.kept.iter().enumerate() {
                xi[p] = block.x.get(r, c);
            }
            // The error variance without the row.
            let sigma2_without = (rss - e * e / (1.0 - h)) / (df - 1.0);
            if sigma2_without.is_nan() || sigma2_without <= 0.0 {
                continue;
            }
            let sigma_without = sigma2_without.sqrt();
            let scale = e / (1.0 - h);
            let drives = (0..k).any(|j| {
                let mut bx = 0.0;
                for (p, x) in xi.iter().enumerate() {
                    bx += bread.get(j, p) * x;
                }
                let dfbetas = bx * scale / (sigma_without * bread.get(j, j).sqrt());
                dfbetas.abs() > DRIVING_DFBETAS
            });
            if drives {
                driving.push(design.rows[row] + 1);
            }
        }
    }
    driving
}

/// R's rule for an essentially perfect fit (`summary.lm`): the residual variance below 1e-30 of
/// the fitted values' scale, their squared mean plus their variance, when the standard errors
/// are rounding and nothing more. No fitted value is no fit.
fn essentially_perfect(sigma: f64, fitted: &[f64]) -> bool {
    if fitted.is_empty() {
        return false;
    }
    let n = fitted.len() as f64;
    let mean = sum(fitted) / n;
    let squares: Vec<f64> = fitted.iter().map(|f| (f - mean) * (f - mean)).collect();
    let variance = if n > 1.0 {
        sum(&squares) / (n - 1.0)
    } else {
        0.0
    };
    sigma * sigma < (mean * mean + variance) * 1e-30
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

#[cfg(test)]
mod tests {
    use super::essentially_perfect;

    /// Fitted values −1, 0, 1 have mean zero and variance one, so the threshold is 1e-30 of the
    /// residual variance: a divisor of the variance other than n − 1 (n + 1, n, or a product)
    /// moves it to 0.5, 0.667 or 4 times that.
    #[test]
    fn the_threshold_is_the_squared_mean_plus_the_variance_over_n_minus_one() {
        let fitted = [-1.0, 0.0, 1.0];
        assert!(essentially_perfect((0.9e-30_f64).sqrt(), &fitted));
        assert!(!essentially_perfect((1.1e-30_f64).sqrt(), &fitted));
        // The mean's share: constant fitted values of 10 have no variance and a squared mean of
        // a hundred.
        let constant = [10.0, 10.0, 10.0];
        assert!(essentially_perfect((99e-30_f64).sqrt(), &constant));
        assert!(!essentially_perfect((101e-30_f64).sqrt(), &constant));
    }

    /// One fitted value has no variance, not a division of zero by zero, and no fitted value
    /// is no fit.
    #[test]
    fn one_fitted_value_has_a_variance_of_zero() {
        assert!(essentially_perfect((3e-30_f64).sqrt(), &[2.0]));
        assert!(!essentially_perfect((5e-30_f64).sqrt(), &[2.0]));
        assert!(!essentially_perfect(0.0, &[]));
    }
}
