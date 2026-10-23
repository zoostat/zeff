//! A fitted model assembled into the shared result value (Language Reference, sections 3.8 and
//! 10.2; question 3 (a) of release 0.4.0): the t table with its p-values and intervals, the fit
//! measures, and the printed summary.

use crate::design::Design;
use crate::estimator::Fitted;
use zeff_core::display::{fixed, p_value_text};
use zeff_core::value::{Align, Distribution, Fit, Summary, Table};
use zeff_diagnostics::count;
use zeff_dist::families;
use zeff_kernels::Matrix;
use zeff_kernels::reduce::sum_of_squares;

/// What the summary names its standard errors.
pub struct ErrorKind<'a> {
    pub word: &'a str,
}

/// The fit value of a linear model: the table of estimates under the variance `vcov`, the fit
/// measures of least squares, and the summary of question 3 (a).
pub fn linear_fit(
    design: &Design,
    fitted: &Fitted,
    vcov: &Matrix,
    method: &str,
    formula: &str,
    se_kind: &str,
    level: f64,
) -> Fit {
    let n = design.n();
    let k = fitted.kept.len();
    let df_residual = n - k;
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
    let quantile = families::quantile(&student, 1.0 - (1.0 - level) / 2.0);
    let intervals: Vec<(f64, f64)> = fitted
        .estimates
        .iter()
        .zip(&std_errors)
        .map(|(b, se)| (b - quantile * se, b + quantile * se))
        .collect();
    // The fit measures: R² against the total sum of squares about the mean with an intercept,
    // about zero without, as R computes them; undefined when the outcome has no variation.
    let y = design.outcome().unwrap_or_default();
    let total = if design.intercept {
        let mean = zeff_kernels::sum(&y) / n as f64;
        let centred: Vec<f64> = y.iter().map(|v| v - mean).collect();
        sum_of_squares(&centred)
    } else {
        sum_of_squares(&y)
    };
    let rss = fitted.residual_sum_of_squares;
    let (r_squared, adjusted) = if total > 0.0 {
        let r2 = 1.0 - rss / total;
        let numerator = if design.intercept { n - 1 } else { n } as f64;
        (
            Some(r2),
            Some(1.0 - (1.0 - r2) * numerator / df_residual as f64),
        )
    } else {
        (None, None)
    };
    let log_likelihood = -0.5
        * n as f64
        * (zeff_numeric::maths::log(2.0 * std::f64::consts::PI)
            + 1.0
            + zeff_numeric::maths::log(rss / n as f64));
    let sigma = (rss / df_residual as f64).sqrt();
    let summary = summary_of(
        design,
        &terms,
        &omitted,
        &fitted.estimates,
        &std_errors,
        &statistics,
        &p_values,
        &intervals,
        method,
        formula,
        se_kind,
        level,
        r_squared,
        adjusted,
    );
    Fit {
        kind: "RegFit",
        method: method.to_string(),
        formula: formula.to_string(),
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
        se_kind: se_kind.to_string(),
        level,
        n,
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
        summary,
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

/// The summary of question 3 (a): `reg · formula`, the N line, the standard-error line, the
/// table of section 10.2, and the fit measures.
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
    method: &str,
    formula: &str,
    se_kind: &str,
    level: f64,
    r_squared: Option<f64>,
    adjusted: Option<f64>,
) -> Summary {
    let mut before = Vec::new();
    let n_line = if design.dropped > 0 {
        format!(
            "N = {} ({} dropped for NA in model variables)",
            count(design.n() as u64),
            rows(design.dropped)
        )
    } else {
        format!("N = {}", count(design.n() as u64))
    };
    before.push(n_line);
    before.push(format!("standard errors: {se_kind}"));
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
    let table = Table {
        title: format!("{method} · {formula}"),
        columns: vec![
            ("term".to_string(), Align::Left),
            ("estimate".to_string(), Align::Right),
            ("std.error".to_string(), Align::Right),
            ("t value".to_string(), Align::Right),
            ("p value".to_string(), Align::Right),
            (level_heading(level), Align::Left),
        ],
        rows: rows_out,
        more: Vec::new(),
    };
    let measure = |x: Option<f64>| x.map_or("NA".to_string(), |v| fixed(v, 3));
    let after = vec![format!(
        "R² {} · adjusted R² {}",
        measure(r_squared),
        measure(adjusted)
    )];
    Summary {
        before,
        table,
        after,
    }
}

fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}
