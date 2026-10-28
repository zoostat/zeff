//! Ordinary least squares on the kernels (Engine and Project Specification, section 8.3, point
//! 10, "Tall-skinny QR"; section 13: "OLS is solved by QR decomposition, never by inverting
//! X′X"; Language Reference, section 7.2, "Collinearity"): a first pass over the design for the
//! column norms; the blocked Householder QR of [X y] with every column scaled to unit norm; the
//! pivoted QR of the k × k R factor with the tolerance 1e-7, which drops a column collinear with
//! the columns before it; the estimates by back substitution, unscaled; and a second pass for the
//! residuals, the fitted values and the leverages, with the self-check that the residuals are
//! orthogonal to the regressors (section 11.1).

use zeff_core::runtime::{CallContext, Fail, fail};
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::qr::{TallSkinnyQr, pivoted_qr};
use zeff_kernels::svd::condition_number;
use zeff_kernels::triangular::{inverse_upper, solve_upper, solve_upper_transposed};
use zeff_kernels::{LaneSum, Matrix, dot, sum_of_squares};
use zeff_results::design::{Block, Design};
use zeff_results::estimator::{Estimator, Fitted};

/// The pivot tolerance of section 7.2: a column whose remaining norm is below this share of its
/// own norm is collinear with the columns before it.
pub const PIVOT_TOLERANCE: f64 = 1e-7;

/// Ordinary least squares.
pub struct Ols;

impl Estimator for Ols {
    fn fit(&self, ctx: &mut CallContext<'_>, design: &Design) -> Result<Fitted, Fail> {
        let k = design.k();
        let n = design.n();
        if n == 0 {
            let (message, help) = if design.dropped == 0 {
                (
                    "no row remains to fit the model: the frame has no rows".to_string(),
                    "a model needs rows; the frame's own steps left none",
                )
            } else {
                (
                    "no row remains to fit the model: every row has NA in a model variable"
                        .to_string(),
                    "`describe` shows each variable's missing count; `where(!is_missing(x))` or `recode` fills the gaps",
                )
            };
            return Err(fail(
                Diagnostic::error("Z0801", message)
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "nothing to estimate")
                    .help(help),
            ));
        }
        // The first pass: each column's norm, which scales it to unit norm; the lane sums run
        // on across the blocks, so the design's block size changes no bit.
        let mut accumulators: Vec<LaneSum> = (0..k).map(|_| LaneSum::new()).collect();
        // The largest size in each column, which tells a zero column from one whose squares
        // underflow or overflow; and the outcome's, which must be squared too.
        let mut largest: Vec<f64> = vec![0.0; k];
        let mut largest_outcome = 0.0_f64;
        for block in design.blocks() {
            for (c, accumulator) in accumulators.iter_mut().enumerate() {
                let column = block.x.column(c);
                accumulator.push_squares(column);
                largest[c] = column.iter().fold(largest[c], |m, x| m.max(x.abs()));
            }
            largest_outcome = block.y.iter().fold(largest_outcome, |m, y| m.max(y.abs()));
        }
        // An outcome beyond the size whose square can be represented has no residual sum of
        // squares, no standard error and no fit measure (the formula fuzz target found it).
        if largest_outcome > OVERSIZED {
            let outcome = design.outcome_name().unwrap_or_default();
            return Err(fail(
                Diagnostic::error(
                    "Z0803",
                    format!(
                        "`{outcome}` has values too large to compute with (above 1e150 in size)"
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "an outcome too large to square")
                .help(format!(
                    "divide the outcome by a power of ten first, `add({outcome}_m = {outcome} / 1e6)`, and read the estimates in those units"
                )),
            ));
        }
        let sums: Vec<f64> = accumulators.iter().map(LaneSum::value).collect();
        // A column whose sum of squares overflowed is scaled to zero, so that the pivoting drops
        // it and it is reported as oversized.
        let scales: Vec<f64> = sums
            .iter()
            .map(|s| {
                if *s > 0.0 && s.is_finite() {
                    1.0 / s.sqrt()
                } else if s.is_infinite() {
                    0.0
                } else {
                    1.0
                }
            })
            .collect();
        // The blocked QR of [X y], X scaled.
        let mut qr = TallSkinnyQr::new(k + 1);
        for block in design.blocks() {
            let rows = block.x.rows();
            let augmented = Matrix::from_fn(rows, k + 1, |r, c| {
                if c < k {
                    // An oversized column (scale zero) may hold an infinity, which the weighted
                    // design can produce from finite factors; zero times it would be not a
                    // number, so the column is zero outright.
                    if scales[c] == 0.0 {
                        0.0
                    } else {
                        block.x.get(r, c) * scales[c]
                    }
                } else {
                    block.y[r]
                }
            });
            qr.push_rows(&augmented);
        }
        let r_full = qr.finish();
        let pivoted = pivoted_qr(&r_full, k, PIVOT_TOLERANCE);
        let rank = pivoted.rank;
        if rank == 0 {
            return Err(fail(
                Diagnostic::error(
                    "Z0802",
                    "no regressor remains: every column of the design is zero",
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "nothing to estimate")
                .help("a model needs at least one regressor that varies, or an intercept"),
            ));
        }
        if n <= rank {
            return Err(fail(
                Diagnostic::error(
                    "Z0802",
                    format!(
                        "no residual degrees of freedom: {} and {} {}",
                        rows_text(n),
                        count(rank as u64),
                        if rank == 1 { "regressor" } else { "regressors" }
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "as many regressors as rows, or more")
                .help("a model needs more rows than regressors; drop terms or use more data"),
            ));
        }
        let r11 = Matrix::from_fn(rank, rank, |i, j| pivoted.r.get(i, j));
        let qty: Vec<f64> = (0..rank).map(|i| pivoted.r.get(i, k)).collect();
        let scaled = solve_upper(&r11, &qty)
            .map_err(|_| internal(ctx, "a zero pivot among the accepted columns"))?;
        let kept: Vec<usize> = pivoted.order[..rank].to_vec();
        // The pivot candidates left out come after the accepted ones; the outcome's column, never
        // a candidate, stays last.
        let mut omitted: Vec<usize> = pivoted.order[rank..k].to_vec();
        omitted.sort_unstable();
        let zero: Vec<usize> = omitted
            .iter()
            .copied()
            .filter(|&c| sums[c] == 0.0 && largest[c] == 0.0)
            .collect();
        let vanishing: Vec<usize> = omitted
            .iter()
            .copied()
            .filter(|&c| sums[c] == 0.0 && largest[c] > 0.0)
            .collect();
        let oversized: Vec<usize> = omitted
            .iter()
            .copied()
            .filter(|&c| !sums[c].is_finite())
            .collect();
        let kept_scales: Vec<f64> = kept.iter().map(|&c| scales[c]).collect();
        let estimates: Vec<f64> = scaled
            .iter()
            .zip(&kept_scales)
            .map(|(b, s)| b * s)
            .collect();
        // The second pass: y − Xβ row by row, the leverage from R, and the products Xᵀe for the
        // self-check.
        let mut residuals = Vec::with_capacity(n);
        let mut fitted = Vec::with_capacity(n);
        let mut leverage = Vec::with_capacity(n);
        let mut xte = vec![0.0; rank];
        let mut row_values = vec![0.0; rank];
        let mut scaled_row = vec![0.0; rank];
        for block in design.blocks() {
            for r in 0..block.x.rows() {
                for (p, &c) in kept.iter().enumerate() {
                    row_values[p] = block.x.get(r, c);
                    scaled_row[p] = row_values[p] * kept_scales[p];
                }
                let yhat = dot(&row_values, &estimates);
                let e = block.y[r] - yhat;
                residuals.push(e);
                fitted.push(yhat);
                let z = solve_upper_transposed(&r11, &scaled_row)
                    .map_err(|_| internal(ctx, "a zero pivot in the leverage"))?;
                leverage.push(sum_of_squares(&z));
                for (acc, x) in xte.iter_mut().zip(&row_values) {
                    *acc += x * e;
                }
            }
        }
        let rss = sum_of_squares(&residuals);
        let condition = condition_number(&r11);
        // Self-check (section 11.1): the residuals are orthogonal to the regressors, to within the
        // rounding of computing them, which grows with the outcome's size and the condition
        // number of the scaled design. The norms are computed at the scale of the values, so
        // that an outcome far below one (whose squares underflow to zero) does not make the
        // bound vanish; the formula fuzz target found that case.
        let residual_norm = scaled_norm(&residuals);
        let outcome_norm = scaled_norm(&design.outcome().unwrap_or_default());
        let slack = (1e-8_f64).max(10.0 * f64::EPSILON * condition);
        for (p, &c) in kept.iter().enumerate() {
            let column_norm = sums[c].sqrt();
            let bound = slack * column_norm * (residual_norm + outcome_norm);
            if xte[p].abs() > bound {
                return Err(internal(
                    ctx,
                    "the residuals are not orthogonal to the regressors",
                ));
            }
        }
        Ok(Fitted {
            kept,
            omitted,
            zero,
            vanishing,
            oversized,
            estimates,
            condition_number: condition,
            r: r11,
            scales: kept_scales,
            residuals,
            fitted,
            leverage,
            residual_sum_of_squares: rss,
        })
    }

    fn scores(&self, fitted: &Fitted, block: &Block, first_row: usize) -> Matrix {
        Matrix::from_fn(block.x.rows(), fitted.kept.len(), |r, p| {
            block.x.get(r, fitted.kept[p]) * fitted.residuals[first_row + r]
        })
    }

    fn bread(&self, fitted: &Fitted) -> Matrix {
        let inverse = inverse_upper(&fitted.r).expect("the accepted pivots are not zero");
        let k = fitted.kept.len();
        // (RᵀR)⁻¹ = R⁻¹ R⁻ᵀ, unscaled by the columns' scales.
        Matrix::from_fn(k, k, |i, j| {
            let mut sum = 0.0;
            for l in 0..k {
                sum += inverse.get(i, l) * inverse.get(j, l);
            }
            sum * fitted.scales[i] * fitted.scales[j]
        })
    }
}

fn rows_text(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the fit that failed its self-check"),
    )
}

/// The size above which a value's square cannot be represented with room for a sum of them, the
/// mirror of the 1e-150 below which a column vanishes: a regressor beyond it is dropped with a
/// note, an outcome beyond it is refused (Z0803).
const OVERSIZED: f64 = 1e150;

/// The Euclidean norm computed at the scale of the largest value, so that it neither underflows
/// nor overflows for values far from one; zero for no values, and the largest value itself when
/// that is not finite.
fn scaled_norm(values: &[f64]) -> f64 {
    let scale = values.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    if scale == 0.0 || !scale.is_finite() {
        return scale;
    }
    let scaled: Vec<f64> = values.iter().map(|v| v / scale).collect();
    scale * sum_of_squares(&scaled).sqrt()
}
