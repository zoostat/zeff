//! A fitted model assembled into the shared result value (Language Reference, sections 3.8 and
//! 10.2; question 3 (a) of release 0.4.0): the t table with its p-values and intervals, the fit
//! measures, and the printed summary.

use crate::design::Design;
use crate::estimator::Fitted;
use std::rc::Rc;
use zeff_core::display::{fixed, fixed_grouped, p_value_text};
use zeff_core::runtime::CallContext;
use zeff_core::value::{Align, Distribution, Fit, FixedEffect, Formula, LikelihoodFit, Table};
use zeff_diagnostics::{Diagnostic, count};
use zeff_dist::families;
use zeff_kernels::Matrix;
use zeff_numeric::order::WeightKind;

/// What the summary says about the fit besides the table: the method, the formula, how the
/// standard errors were computed (as the summary's line writes it), the degrees of freedom of
/// the t table, the confidence level, the observation count printed, what its line says in
/// brackets, and further lines before the grid (the weights).
pub struct Setting<'a> {
    pub method: &'a str,
    pub formula: &'a str,
    pub se_line: &'a str,
    pub df: usize,
    pub level: f64,
    /// The observation count: the rows used, or the sum of the frequency weights.
    pub n: usize,
    pub n_note: Option<String>,
    pub more_lines: Vec<String>,
    /// The formula as a value, the category levels of the design and the rows of the frame,
    /// kept in the fit for `predict`, `residuals` and `fitted`.
    pub model: Formula,
    pub levels: Vec<(String, Vec<String>)>,
    pub frame_rows: usize,
    /// The fixed effects absorbed, with their levels' estimates, as written (`fe: firm + year`
    /// in the title, `fixed effects: firm (397), year (12)` before the grid); empty without
    /// `fe:`.
    pub effects: Vec<FixedEffect>,
    /// The parameters the absorbed effects stand for in the degrees of freedom.
    pub absorbed_parameters: usize,
    /// The rows dropped as singletons of the effects.
    pub singletons: usize,
}

/// The fit value of a linear model: the table of estimates under the variance `vcov`, the fit
/// measures of least squares, and the summary of question 3 (a).
pub fn linear_fit(design: &Design, fitted: &Fitted, vcov: &Matrix, setting: &Setting<'_>) -> Fit {
    let n = design.n();
    let k = fitted.kept.len();
    let absorbed = setting.absorbed_parameters;
    let df_residual = setting.df;
    let terms: Vec<String> = fitted
        .kept
        .iter()
        .map(|&c| design.names[c].clone())
        .collect();
    let omitted: Vec<String> = fitted
        .omitted
        .iter()
        .map(|&c| design.names[c].clone())
        .collect();
    let std_errors: Vec<f64> = (0..k).map(|i| vcov.get(i, i).sqrt()).collect();
    let statistics: Vec<f64> = fitted
        .estimates
        .iter()
        .zip(&std_errors)
        .map(|(b, se)| b / se)
        .collect();
    let student = Distribution::Family {
        name: "t",
        params: vec![("df", df_residual as f64)],
    };
    let p_values: Vec<f64> = statistics
        .iter()
        .map(|t| 2.0 * families::cdf(&student, t.abs()).upper)
        .collect();
    let quantile = families::quantile(&student, 1.0 - (1.0 - setting.level) / 2.0);
    let intervals: Vec<(f64, f64)> = fitted
        .estimates
        .iter()
        .zip(&std_errors)
        .map(|(b, se)| (b - quantile * se, b + quantile * se))
        .collect();
    // The fit measures: R² against the total sum of squares about the (weighted) mean with an
    // intercept, about zero without, as R computes them; undefined when the outcome has no
    // variation. N is the rows used, or the sum of the frequency weights.
    let y = design.outcome().unwrap_or_default();
    let weights: Vec<f64> = design.weights.clone().unwrap_or_else(|| vec![1.0; n]);
    let n_obs = setting.n as f64;
    let total = if design.intercept {
        let weighted: Vec<f64> = y.iter().zip(&weights).map(|(v, w)| v * w).collect();
        let mean = zeff_kernels::sum(&weighted) / zeff_kernels::sum(&weights);
        let centred: Vec<f64> = y
            .iter()
            .zip(&weights)
            .map(|(v, w)| (v - mean) * (v - mean) * w)
            .collect();
        zeff_kernels::sum(&centred)
    } else {
        let squares: Vec<f64> = y.iter().zip(&weights).map(|(v, w)| v * v * w).collect();
        zeff_kernels::sum(&squares)
    };
    let rss = fitted.residual_sum_of_squares;
    let residual_df = (n_obs - k as f64 - absorbed as f64).max(1.0);
    let (mut r_squared, mut adjusted) = if total > 0.0 {
        let r2 = 1.0 - rss / total;
        let numerator = if design.intercept { n_obs - 1.0 } else { n_obs };
        (Some(r2), Some(1.0 - (1.0 - r2) * numerator / residual_df))
    } else {
        (None, None)
    };
    // With fixed effects the measures above are the within forms, on the demeaned outcome; the
    // overall forms take the original outcome about its weighted mean with the effects' fit
    // included, as fixest's `r2` and `ar2`, and the adjusted within form counts the effects'
    // parameters as fixest's `war2` does: 1 − (1 − R²w)(N − p)/(N − k − p).
    let (r_squared_within, adjusted_r_squared_within) = match &design.original_outcome {
        Some(original) => {
            let within = r_squared;
            let adjusted_within =
                within.map(|w| 1.0 - (1.0 - w) * (n_obs - absorbed as f64) / residual_df);
            let weighted: Vec<f64> = original.iter().zip(&weights).map(|(v, w)| v * w).collect();
            let mean = zeff_kernels::sum(&weighted) / zeff_kernels::sum(&weights);
            let centred: Vec<f64> = original
                .iter()
                .zip(&weights)
                .map(|(v, w)| (v - mean) * (v - mean) * w)
                .collect();
            let total = zeff_kernels::sum(&centred);
            if total > 0.0 {
                let r2 = 1.0 - rss / total;
                r_squared = Some(r2);
                adjusted = Some(1.0 - (1.0 - r2) * (n_obs - 1.0) / residual_df);
            } else {
                r_squared = None;
                adjusted = None;
            }
            (within, adjusted_within)
        }
        None => (None, None),
    };
    // The log-likelihood as R's `logLik` gives it: under analytic and sampling weights, which R
    // takes as precision weights, with half the sum of the logarithms of the weights (as scaled
    // to average one, which leaves R's value unchanged); under frequency weights the rows stand
    // repeated and the sum of squares is already theirs.
    let weight_term = match (design.weight_kind, &design.weights) {
        (Some(WeightKind::Analytic | WeightKind::Sampling), Some(w)) => {
            let logs: Vec<f64> = w.iter().map(|&x| zeff_numeric::maths::log(x)).collect();
            0.5 * zeff_kernels::sum(&logs)
        }
        _ => 0.0,
    };
    let log_likelihood = weight_term
        - 0.5
            * n_obs
            * (zeff_numeric::maths::log(2.0 * std::f64::consts::PI)
                + 1.0
                + zeff_numeric::maths::log(rss / n_obs));
    let sigma = (rss / residual_df).sqrt();
    let (term_sd, outcome_sd) = deviations(design, fitted);
    // The Gram matrix and its inverse from the scaled R factor: XᵀX = D⁻¹ RᵀR D⁻¹ and
    // (XᵀX)⁻¹ = D R⁻¹ R⁻ᵀ D, with D the kept columns' scales.
    let gram: Vec<f64> = (0..k)
        .flat_map(|i| (0..k).map(move |j| (i, j)))
        .map(|(i, j)| {
            let mut sum = 0.0;
            for l in 0..k {
                sum += fitted.r.get(l, i) * fitted.r.get(l, j);
            }
            sum / (fitted.scales[i] * fitted.scales[j])
        })
        .collect();
    let inverse_r = zeff_kernels::triangular::inverse_upper(&fitted.r)
        .expect("the accepted pivots are not zero");
    let inverse_gram: Vec<f64> = (0..k)
        .flat_map(|i| (0..k).map(move |j| (i, j)))
        .map(|(i, j)| {
            let mut sum = 0.0;
            for l in 0..k {
                sum += inverse_r.get(i, l) * inverse_r.get(j, l);
            }
            sum * fitted.scales[i] * fitted.scales[j]
        })
        .collect();
    let (title, after) = linear_title_and_measures(setting, r_squared, adjusted, r_squared_within);
    let summary = summary_of(
        design,
        &terms,
        &omitted,
        &fitted.estimates,
        &std_errors,
        &statistics,
        &p_values,
        &intervals,
        setting,
        Lines {
            title,
            statistic: "t value",
            after: vec![after],
        },
    );
    Fit {
        kind: "RegFit",
        name: None,
        source: None,
        method: setting.method.to_string(),
        formula: setting.formula.to_string(),
        terms,
        omitted,
        estimates: fitted.estimates.clone(),
        std_errors,
        statistics,
        p_values,
        intervals,
        vcov: (0..k)
            .flat_map(|i| (0..k).map(move |j| (i, j)))
            .map(|(i, j)| vcov.get(i, j))
            .collect(),
        se_kind: setting.se_line.to_string(),
        level: setting.level,
        n: setting.n,
        dropped: design.dropped,
        dropped_in: design.dropped_in.clone(),
        df_residual,
        rows: design.rows.clone(),
        residuals: fitted.residuals.clone(),
        fitted: fitted.fitted.clone(),
        leverage: fitted.leverage.clone(),
        r_squared,
        adjusted_r_squared: adjusted,
        log_likelihood,
        residual_sum_of_squares: rss,
        sigma,
        condition_number: fitted.condition_number,
        notes: design.notes.clone(),
        warnings: Vec::new(),
        term_sd,
        outcome_sd,
        summary,
        model: setting.model.clone(),
        levels: setting.levels.clone(),
        frame_rows: setting.frame_rows,
        weights: design.weights.clone(),
        weight_kind: design.weight_kind,
        gram,
        inverse_gram,
        effects: setting.effects.clone(),
        singletons: setting.singletons,
        absorbed_parameters: absorbed,
        r_squared_within,
        adjusted_r_squared_within,
        instrumented: None,
        likelihood: None,
    }
}

/// The fit value of a model fitted by maximum likelihood (release 0.9.0, step 5a): the z table
/// of the estimates under the variance `vcov`, normal p-values and intervals, and under the
/// table the log-likelihood, AIC and McFadden's R², then the deviance against the null
/// deviance and how many iterations the optimiser took. `log_likelihood` and `null` are the
/// log-likelihoods of the model and of the intercept alone.
pub fn likelihood_fit(
    design: &Design,
    fitted: &Fitted,
    vcov: &Matrix,
    setting: &Setting<'_>,
    likelihood: LikelihoodFit,
    log_likelihood: f64,
    null: f64,
) -> Fit {
    let k = fitted.kept.len();
    let terms: Vec<String> = fitted
        .kept
        .iter()
        .map(|&c| design.names[c].clone())
        .collect();
    let omitted: Vec<String> = fitted
        .omitted
        .iter()
        .map(|&c| design.names[c].clone())
        .collect();
    let std_errors: Vec<f64> = (0..k).map(|i| vcov.get(i, i).sqrt()).collect();
    let statistics: Vec<f64> = fitted
        .estimates
        .iter()
        .zip(&std_errors)
        .map(|(b, se)| b / se)
        .collect();
    let p_values: Vec<f64> = statistics
        .iter()
        .map(|z| (2.0 * zeff_numeric::special::normal_tails(z.abs()).upper).min(1.0))
        .collect();
    let q = zeff_numeric::special::normal_quantile((1.0 - setting.level) / 2.0, true);
    let intervals: Vec<(f64, f64)> = fitted
        .estimates
        .iter()
        .zip(&std_errors)
        .map(|(b, se)| (b - q * se, b + q * se))
        .collect();
    let aic = -2.0 * log_likelihood + 2.0 * k as f64;
    let mcfadden = if null < 0.0 {
        1.0 - log_likelihood / null
    } else {
        f64::NAN
    };
    let after = vec![
        format!(
            "{} {} · AIC {} · McFadden R² {}",
            if likelihood.pseudo {
                "log pseudo-likelihood"
            } else {
                "log-likelihood"
            },
            fixed_grouped(log_likelihood, 2),
            fixed_grouped(aic, 2),
            fixed(mcfadden, 3)
        ),
        format!(
            "deviance {} on {} df · null deviance {} on {} df",
            fixed_grouped(likelihood.deviance, 2),
            count(likelihood.df_deviance as u64),
            fixed_grouped(likelihood.null_deviance, 2),
            count(likelihood.df_null as u64),
        ),
        format!(
            "{} {} {} of {}",
            if likelihood.converged {
                "converged in"
            } else {
                "stopped after"
            },
            likelihood.iterations,
            if likelihood.iterations == 1 {
                "iteration"
            } else {
                "iterations"
            },
            likelihood.method
        ),
    ];
    let title = format!(
        "{} · {} · {}, {}",
        setting.method, setting.formula, likelihood.family, likelihood.link
    );
    let summary = summary_of(
        design,
        &terms,
        &omitted,
        &fitted.estimates,
        &std_errors,
        &statistics,
        &p_values,
        &intervals,
        setting,
        Lines {
            title,
            statistic: "z value",
            after,
        },
    );
    let (term_sd, outcome_sd) = deviations(design, fitted);
    let gram = |inverse: bool| -> Vec<f64> {
        let factor = if inverse {
            zeff_kernels::triangular::inverse_upper(&fitted.r)
                .expect("the accepted pivots are not zero")
        } else {
            fitted.r.clone()
        };
        (0..k)
            .flat_map(|i| (0..k).map(move |j| (i, j)))
            .map(|(i, j)| {
                let mut sum = 0.0;
                for l in 0..k {
                    sum += if inverse {
                        factor.get(i, l) * factor.get(j, l)
                    } else {
                        factor.get(l, i) * factor.get(l, j)
                    };
                }
                if inverse {
                    sum * fitted.scales[i] * fitted.scales[j]
                } else {
                    sum / (fitted.scales[i] * fitted.scales[j])
                }
            })
            .collect()
    };
    let df_residual = likelihood.df_deviance;
    Fit {
        kind: "GlmFit",
        name: None,
        source: None,
        method: setting.method.to_string(),
        formula: setting.formula.to_string(),
        terms,
        omitted,
        estimates: fitted.estimates.clone(),
        std_errors,
        statistics,
        p_values,
        intervals,
        vcov: (0..k)
            .flat_map(|i| (0..k).map(move |j| (i, j)))
            .map(|(i, j)| vcov.get(i, j))
            .collect(),
        se_kind: setting.se_line.to_string(),
        level: setting.level,
        n: setting.n,
        dropped: design.dropped,
        dropped_in: design.dropped_in.clone(),
        df_residual,
        rows: design.rows.clone(),
        residuals: fitted.residuals.clone(),
        fitted: fitted.fitted.clone(),
        leverage: fitted.leverage.clone(),
        r_squared: None,
        adjusted_r_squared: None,
        log_likelihood,
        // The deviance stands where a least-squares fit keeps its residual sum of squares, and
        // the dispersion of the binomial family, one, where it keeps σ.
        residual_sum_of_squares: likelihood.deviance,
        sigma: 1.0,
        condition_number: fitted.condition_number,
        notes: design.notes.clone(),
        warnings: Vec::new(),
        term_sd,
        outcome_sd,
        summary,
        model: setting.model.clone(),
        levels: setting.levels.clone(),
        frame_rows: setting.frame_rows,
        weights: design.weights.clone(),
        weight_kind: design.weight_kind,
        // The information XᵀWX at the solution, and its inverse.
        gram: gram(false),
        inverse_gram: gram(true),
        effects: Vec::new(),
        singletons: 0,
        absorbed_parameters: 0,
        r_squared_within: None,
        adjusted_r_squared_within: None,
        instrumented: None,
        likelihood: Some(Rc::new(likelihood)),
    }
}

/// The notes for the columns a fit dropped, each kind in its own words (section 7.2,
/// "Collinearity"): a column the fixed effects absorbed (`absorbed_columns`, with the effects as
/// written), a category level with no observations among the rows used, a column that is zero
/// in every row, a column whose values are too small or too large to be scaled, and the rest
/// collinear with the columns before them. Each is printed and kept in `notes`.
pub fn dropped_notes(
    ctx: &mut CallContext<'_>,
    notes: &mut Vec<String>,
    design: &Design,
    fitted: &Fitted,
    absorbed_columns: &[usize],
    effects_text: &str,
) {
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
    // A column the fixed effects absorbed (section 7.2, "Collinearity") is zeroed by the
    // projection and dropped by the fit; it is named so, and not as a zero column.
    note(
        ctx,
        notes,
        names(absorbed_columns),
        &format!("is absorbed by fe: {effects_text} and was dropped"),
        &format!("are absorbed by fe: {effects_text} and were dropped"),
    );
    let (levels, zeros): (Vec<usize>, Vec<usize>) = fitted
        .zero
        .iter()
        .filter(|c| !absorbed_columns.contains(c))
        .partition(|&&c| design.names[c].contains('='));
    note(
        ctx,
        notes,
        names(&levels),
        "has no observations among the rows used and was dropped",
        "have no observations among the rows used and were dropped",
    );
    note(
        ctx,
        notes,
        names(&zeros),
        "is zero in every row used and was dropped",
        "are zero in every row used and were dropped",
    );
    note(
        ctx,
        notes,
        names(&fitted.vanishing),
        "has values too small to be scaled (below 1e-150 in size) and was dropped",
        "have values too small to be scaled (below 1e-150 in size) and were dropped",
    );
    note(
        ctx,
        notes,
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
        notes,
        names(&collinear),
        "is collinear with the columns before it and was dropped",
        "are collinear with the columns before them and were dropped",
    );
}

/// The confidence level as a column heading writes it: `95% CI`, `99.9% CI`.
fn level_heading(level: f64) -> String {
    let percent = level * 100.0;
    let text = if (percent - percent.round()).abs() < 1e-9 {
        format!("{}", percent.round() as i64)
    } else {
        fixed(percent, 1)
    };
    format!("{text}% CI")
}

/// The title of a fit's summary, the heading of its statistic and the lines under its table.
struct Lines {
    title: String,
    statistic: &'static str,
    after: Vec<String>,
}

/// The title of a least-squares fit, `reg · formula` (with `· fe: ...` under fixed effects),
/// and the line of its fit measures under the table.
fn linear_title_and_measures(
    setting: &Setting<'_>,
    r_squared: Option<f64>,
    adjusted: Option<f64>,
    r_squared_within: Option<f64>,
) -> (String, String) {
    let measure = |x: Option<f64>| x.map_or("NA".to_string(), |v| fixed(v, 3));
    if setting.effects.is_empty() {
        (
            format!("{} · {}", setting.method, setting.formula),
            format!(
                "R² {} · adjusted R² {}",
                measure(r_squared),
                measure(adjusted)
            ),
        )
    } else {
        (
            format!(
                "{} · {} · fe: {}",
                setting.method,
                setting.formula,
                setting
                    .effects
                    .iter()
                    .map(|e| e.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" + ")
            ),
            format!(
                "R² within {} · R² overall {}",
                measure(r_squared_within),
                measure(r_squared)
            ),
        )
    }
}

/// The summary of question 3 (a): the title, the N line, the weights line when weighted, the
/// standard-error line, the table of section 10.2, and the lines under it.
#[allow(clippy::too_many_arguments)]
fn summary_of(
    design: &Design,
    terms: &[String],
    omitted: &[String],
    estimates: &[f64],
    std_errors: &[f64],
    statistics: &[f64],
    p_values: &[f64],
    intervals: &[(f64, f64)],
    setting: &Setting<'_>,
    lines: Lines,
) -> Table {
    let mut before = Vec::new();
    let n_line = match &setting.n_note {
        Some(note) => format!("N = {} ({note})", count(setting.n as u64)),
        None => format!("N = {}", count(setting.n as u64)),
    };
    before.push(n_line);
    if !setting.effects.is_empty() {
        before.push(format!(
            "fixed effects: {}",
            setting
                .effects
                .iter()
                .map(|e| format!("{} ({})", e.name, count(e.keys.len() as u64)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    before.extend(setting.more_lines.iter().cloned());
    before.push(format!("standard errors: {}", setting.se_line));
    // The rows in the design's order: an omitted column keeps its place.
    let mut rows_out = Vec::new();
    let mut kept_at = 0;
    for name in &design.names {
        if omitted.contains(name) {
            rows_out.push(vec![
                name.clone(),
                "(omitted)".to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ]);
            continue;
        }
        let i = kept_at;
        kept_at += 1;
        debug_assert_eq!(&terms[i], name);
        rows_out.push(vec![
            name.clone(),
            fixed(estimates[i], 4),
            fixed(std_errors[i], 4),
            fixed(statistics[i], 2),
            p_value_text(p_values[i]),
            format!(
                "[{}, {}]",
                fixed(intervals[i].0, 4),
                fixed(intervals[i].1, 4)
            ),
        ]);
    }
    Table {
        title: lines.title,
        before,
        columns: vec![
            ("term".to_string(), Align::Left),
            ("estimate".to_string(), Align::Right),
            ("std.error".to_string(), Align::Right),
            (lines.statistic.to_string(), Align::Right),
            ("p value".to_string(), Align::Right),
            (level_heading(setting.level), Align::Left),
        ],
        rows: rows_out,
        after: lines.after,
        more: Vec::new(),
        rules: Vec::new(),
    }
}

/// The standard deviation of each kept column and of the outcome over the rows used, for the
/// standardized coefficients of question 3 (d): under frequency weights each row counts as its
/// weight, the divisor N − 1 with N their sum, as R computes them on the replicated rows; under
/// analytic and sampling weights, and without weights, each row counts once, as R's `sd` of the
/// model matrix does. Two passes over the blocks, the rows unscaled by their weights.
pub fn deviations(design: &Design, fitted: &Fitted) -> (Vec<f64>, f64) {
    let kept = &fitted.kept;
    let n = design.n();
    let weights = design.weights.as_ref();
    let frequency = design.weight_kind == Some(WeightKind::Frequency);
    let count_of = |row: usize| -> f64 {
        if frequency {
            weights.map_or(1.0, |w| w[row])
        } else {
            1.0
        }
    };
    let root_of = |row: usize| -> f64 { weights.map_or(1.0, |w| w[row].sqrt()) };
    let total: f64 = (0..n).map(count_of).sum();
    let mut sums = vec![0.0; kept.len()];
    for block in design.blocks() {
        for r in 0..block.x.rows() {
            let row = block.start + r;
            let (count, root) = (count_of(row), root_of(row));
            for (p, &c) in kept.iter().enumerate() {
                sums[p] += count * block.x.get(r, c) / root;
            }
        }
    }
    let means: Vec<f64> = sums.iter().map(|s| s / total).collect();
    let mut squares = vec![0.0; kept.len()];
    for block in design.blocks() {
        for r in 0..block.x.rows() {
            let row = block.start + r;
            let (count, root) = (count_of(row), root_of(row));
            for (p, &c) in kept.iter().enumerate() {
                let d = block.x.get(r, c) / root - means[p];
                squares[p] += count * d * d;
            }
        }
    }
    let sd = |square: f64| -> f64 {
        if total > 1.0 {
            (square / (total - 1.0)).sqrt()
        } else {
            0.0
        }
    };
    let term_sd: Vec<f64> = squares.iter().map(|&s| sd(s)).collect();
    let outcome_sd = match design.outcome() {
        Some(y) => {
            let mean: f64 = (0..n).map(|r| count_of(r) * y[r]).sum::<f64>() / total;
            sd((0..n)
                .map(|r| count_of(r) * (y[r] - mean) * (y[r] - mean))
                .sum())
        }
        None => 0.0,
    };
    (term_sd, outcome_sd)
}

/// `228 rows dropped for NA in model variables; 12 rows with zero weight left out`, or `None`
/// with nothing to say.
pub fn n_note(design: &Design) -> Option<String> {
    let mut parts = Vec::new();
    if design.dropped > 0 {
        parts.push(format!(
            "{} dropped for NA in model variables",
            rows(design.dropped)
        ));
    }
    if design.zero_weights > 0 {
        parts.push(format!(
            "{} with zero weight left out",
            rows(design.zero_weights)
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

/// `3 rows`, `1 row`.
pub fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}
