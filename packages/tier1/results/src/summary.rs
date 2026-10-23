//! A fitted model assembled into the shared result value (Language Reference, sections 3.8 and
//! 10.2; question 3 (a) of release 0.4.0): the t table with its p-values and intervals, the fit
//! measures, and the printed summary.

use crate::design::Design;
use crate::estimator::Fitted;
use zeff_core::display::{fixed, p_value_text};
use zeff_core::value::{Align, Distribution, Fit, Formula, Table};
use zeff_diagnostics::count;
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
}

/// The fit value of a linear model: the table of estimates under the variance `vcov`, the fit
/// measures of least squares, and the summary of question 3 (a).
pub fn linear_fit(design: &Design, fitted: &Fitted, vcov: &Matrix, setting: &Setting<'_>) -> Fit {
    let n = design.n();
    let k = fitted.kept.len();
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
    let residual_df = (n_obs - k as f64).max(1.0);
    let (r_squared, adjusted) = if total > 0.0 {
        let r2 = 1.0 - rss / total;
        let numerator = if design.intercept { n_obs - 1.0 } else { n_obs };
        (Some(r2), Some(1.0 - (1.0 - r2) * numerator / residual_df))
    } else {
        (None, None)
    };
    let log_likelihood = -0.5
        * n_obs
        * (zeff_numeric::maths::log(2.0 * std::f64::consts::PI)
            + 1.0
            + zeff_numeric::maths::log(rss / n_obs));
    let sigma = (rss / residual_df).sqrt();
    let (term_sd, outcome_sd) = deviations(design, fitted);
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
        r_squared,
        adjusted,
    );
    Fit {
        kind: "RegFit",
        name: None,
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
        term_sd,
        outcome_sd,
        summary,
        model: setting.model.clone(),
        levels: setting.levels.clone(),
        frame_rows: setting.frame_rows,
        weights: design.weights.clone(),
        weight_kind: design.weight_kind,
    }
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

/// The summary of question 3 (a): `reg · formula`, the N line, the weights line when weighted,
/// the standard-error line, the table of section 10.2, and the fit measures.
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
    r_squared: Option<f64>,
    adjusted: Option<f64>,
) -> Table {
    let mut before = Vec::new();
    let n_line = match &setting.n_note {
        Some(note) => format!("N = {} ({note})", count(setting.n as u64)),
        None => format!("N = {}", count(setting.n as u64)),
    };
    before.push(n_line);
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
    let measure = |x: Option<f64>| x.map_or("NA".to_string(), |v| fixed(v, 3));
    Table {
        title: format!("{} · {}", setting.method, setting.formula),
        before,
        columns: vec![
            ("term".to_string(), Align::Left),
            ("estimate".to_string(), Align::Right),
            ("std.error".to_string(), Align::Right),
            ("t value".to_string(), Align::Right),
            ("p value".to_string(), Align::Right),
            (level_heading(setting.level), Align::Left),
        ],
        rows: rows_out,
        after: vec![format!(
            "R² {} · adjusted R² {}",
            measure(r_squared),
            measure(adjusted)
        )],
        more: Vec::new(),
    }
}

/// The standard deviation of each kept column and of the outcome over the rows used, for the
/// standardized coefficients of question 3 (d): under frequency weights each row counts as its
/// weight, the divisor N − 1 with N their sum, as R computes them on the replicated rows; under
/// analytic and sampling weights, and without weights, each row counts once, as R's `sd` of the
/// model matrix does. Two passes over the blocks, the rows unscaled by their weights.
fn deviations(design: &Design, fitted: &Fitted) -> (Vec<f64>, f64) {
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
