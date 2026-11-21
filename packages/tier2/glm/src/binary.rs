//! `glm` with `family: binomial` (open decision #112; release 0.9.0, step 5a): the design of the
//! formula on the frame, as `reg` builds it; the columns dropped by the collinearity rule of
//! section 7.2 of the Language Reference, as `reg` drops them; the maximum of the binomial
//! likelihood under the link by the shared optimiser of the kernels (Fisher scoring, from R's
//! start); the classical variance, the inverse of the expected information at the maximum; and
//! the fit value with its z table and printed summary.

use std::rc::Rc;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{FitSource, LikelihoodFit, TermKind, Value};
use zeff_diagnostics::{Diagnostic, count};
use zeff_kernels::irls::{Control, IndexModel, Method, Stop, irls};
use zeff_kernels::qr::{TallSkinnyQr, pivoted_qr};
use zeff_kernels::svd::condition_number;
use zeff_kernels::triangular::{inverse_upper, solve_upper_transposed};
use zeff_kernels::{LaneSum, Matrix, sum};
use zeff_results::design::Design;
use zeff_results::estimator::Fitted;
use zeff_results::likelihood::{Binomial, Likelihood, Link};
use zeff_results::summary::{self, Setting, dropped_notes, n_note};

/// The places of the arguments, as `functions()` declares them.
const FAMILY: usize = 2;
const LINK: usize = 3;
const LEVEL: usize = 4;

/// The pivot tolerance of section 7.2, the one `reg` uses: a column whose remaining norm is below
/// this share of its own norm is collinear with the columns before it.
const PIVOT_TOLERANCE: f64 = 1e-7;

/// Values beyond these sizes cannot be squared, or their squares are lost below the smallest
/// double, so no norm scales the column (the rule of `reg`'s least squares).
const OVERSIZED: f64 = 1e150;

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script (glm: {what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the fit that failed its self-check"),
    )
}

/// The columns of the design the fit keeps, and those it drops by kind, as `reg` sorts them:
/// zero in every row used, too small or too large to scale, and collinear with the columns
/// before them.
struct Columns {
    kept: Vec<usize>,
    omitted: Vec<usize>,
    zero: Vec<usize>,
    vanishing: Vec<usize>,
    oversized: Vec<usize>,
}

fn columns(design: &Design) -> Columns {
    let k = design.k();
    let x = design.matrix();
    let (mut zero, mut vanishing, mut oversized, mut candidates) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut scales = Vec::new();
    for c in 0..k {
        let column = x.column(c);
        let largest = column.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        if largest == 0.0 {
            zero.push(c);
            continue;
        }
        if largest > OVERSIZED || !largest.is_finite() {
            oversized.push(c);
            continue;
        }
        let mut lanes = LaneSum::new();
        lanes.push_squares(column);
        let squares = lanes.value();
        if squares == 0.0 {
            vanishing.push(c);
            continue;
        }
        candidates.push(c);
        scales.push(1.0 / squares.sqrt());
    }
    let mut qr = TallSkinnyQr::new(candidates.len());
    let mut row = vec![0.0; candidates.len()];
    for i in 0..x.rows() {
        for (slot, (&c, &s)) in row.iter_mut().zip(candidates.iter().zip(&scales)) {
            *slot = x.get(i, c) * s;
        }
        qr.push_row(&row);
    }
    let r = qr.finish();
    let pivoted = pivoted_qr(&r, candidates.len(), PIVOT_TOLERANCE);
    let mut kept: Vec<usize> = pivoted.order[..pivoted.rank]
        .iter()
        .map(|&p| candidates[p])
        .collect();
    kept.sort_unstable();
    let mut omitted: Vec<usize> = (0..k).filter(|c| !kept.contains(c)).collect();
    omitted.sort_unstable();
    Columns {
        kept,
        omitted,
        zero,
        vanishing,
        oversized,
    }
}

fn rows_text(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

/// A change of the log-likelihood as the message prints it: two decimals, or two significant
/// digits in the exponent form for a change below one hundredth.
fn change_text(change: f64) -> String {
    if change.abs() >= 0.01 {
        format!("{change:.2}")
    } else {
        format!("{change:.1e}")
    }
}

/// `data.glm(formula, family:, link:, level:)`.
pub fn glm(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    glm_with(ctx, args, &Control::default())
}

/// `glm` under the optimiser's settings `control`: the function's are the defaults; a test
/// lowers the iteration cap to reach the refusal of a fit that has not converged.
pub fn glm_with(ctx: &mut CallContext<'_>, args: Args, control: &Control) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(ctx, "called without a frame"));
    };
    let Some(Value::Formula(formula)) = args.get(1) else {
        return Err(internal(ctx, "called without a formula"));
    };
    let (frame, formula) = (frame.clone(), formula.clone());
    // `family:` has no default: a model left without one would be a linear model, which `reg`
    // fits (open decision #112).
    let family = match args.get(FAMILY) {
        Some(Value::Word(word)) => word.clone(),
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`glm` needs `family:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`family:` is missing")
                    .help("a binary outcome is `family: binomial`; a linear model is `reg`"),
            ));
        }
    };
    let link = match args.get(LINK) {
        Some(Value::Word(word)) if word == "probit" => Link::Probit,
        _ => Link::Logit,
    };
    let level = match args.get(LEVEL) {
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
            .primary(ctx.span_of(LEVEL), "not a confidence level")
            .help("the usual levels are 0.9, 0.95 and 0.99"),
        ));
    }

    // A category on the left of `~` is refused here rather than by the design, so that the
    // help writes the 0/1 outcome `glm` takes.
    if let Some(term) = &formula.outcome
        && let TermKind::Var(name) = &term.kind
        && let Some(column) = frame.table.column(name)
        && let Some(levels) = column.levels()
    {
        let level = levels.last().copied().unwrap_or("level").to_string();
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "the outcome `{name}` is a category, and `glm` with `family: {family}` needs 0 and 1, or true and false"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "a category on the left of `~`")
            .help(format!(
                "say which level is the 1: `add({name}_1 = {name} == \"{level}\")` and fit `{name}_1`"
            )),
        ));
    }

    let design = Design::build(ctx, &frame, &formula, &[])?;
    let n = design.n();
    if n == 0 {
        let message = if design.dropped == 0 {
            "no row remains to fit the model: the frame has no rows"
        } else {
            "no row remains to fit the model: every row has NA in a model variable"
        };
        return Err(fail(
            Diagnostic::error("Z0801", message)
                .in_file(ctx.script)
                .primary(ctx.callee_span, "nothing to estimate")
                .help("`describe` shows each variable's missing count"),
        ));
    }
    let y = design
        .outcome()
        .ok_or_else(|| internal(ctx, "a design without an outcome"))?;
    let outcome = design.outcome_name().unwrap_or_default();
    if let Some(bad) = y.iter().find(|v| **v != 0.0 && **v != 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`glm` with `family: {family}` needs an outcome of 0 and 1, or true and false, but `{outcome}` holds {bad}"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "not a binary outcome")
            .help(format!(
                "a yes-or-no outcome is coded 0 and 1 or as a Bool: `add({outcome}_yes = {outcome} == ...)`; a numeric outcome is a linear model, `reg`"
            )),
        ));
    }
    for note in &design.notes {
        ctx.message(Diagnostic::plain_note(note.clone()));
    }

    // The columns kept, and the design restricted to them.
    let chosen = columns(&design);
    if chosen.kept.is_empty() {
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
    if n <= chosen.kept.len() {
        let rank = chosen.kept.len();
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
    // An outcome with one value in every row used has its probability estimated at 0 or 1
    // whatever the regressors: the intercept runs off to infinity and no maximum exists.
    if y.iter().all(|v| *v == y[0]) {
        let (value, probability) = if y[0] == 1.0 { (1, 1) } else { (0, 0) };
        return Err(fail(
            Diagnostic::error(
                "Z0803",
                format!(
                    "glm: `{outcome}` is {value} in every row used, so maximum likelihood does not exist"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "an outcome that does not vary")
            .help(format!(
                "the probability of a 1 would be {probability} whatever the regressors; check how `{outcome}` was coded and which rows were dropped for NA"
            )),
        ));
    }
    let full = design.matrix();
    let x = Matrix::from_fn(n, chosen.kept.len(), |i, j| full.get(i, chosen.kept[j]));
    let model = Binomial::new(y.clone(), link);
    let start: Vec<f64> = (0..n).map(|row| model.start(row)).collect();
    let solution = irls(&x, &model, &start, control).map_err(|stop| {
        let at = match stop {
            Stop::NotFinite { iteration } | Stop::RankLost { iteration, .. } => iteration,
        };
        fail(
            Diagnostic::error(
                "Z0803",
                format!("glm: the estimates diverge at iteration {at}; maximum likelihood does not exist"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "estimates diverge")
            .help(format!("a regressor may predict `{outcome}` perfectly; check how `{outcome}` was coded")),
        )
    })?;

    // The fit in the form every result reads: the estimates, the R factor of the information
    // and its scales, the fitted probabilities, the response residuals and each row's leverage
    // in the weighted fit, h = w xᵀ(XᵀWX)⁻¹x.
    let k = chosen.kept.len();
    let fitted_values: Vec<f64> = solution.eta.iter().map(|&eta| model.mean(eta)).collect();
    let residuals: Vec<f64> = y.iter().zip(&fitted_values).map(|(a, b)| a - b).collect();
    let mut leverage = Vec::with_capacity(n);
    for (row, &eta) in solution.eta.iter().enumerate() {
        let w = model.at(row, eta, false).weight;
        let scaled: Vec<f64> = (0..k).map(|j| x.get(row, j) * solution.scales[j]).collect();
        let solved = solve_upper_transposed(&solution.r, &scaled)
            .map_err(|_| internal(ctx, "a zero pivot in the information"))?;
        let squares: Vec<f64> = solved.iter().map(|v| v * v).collect();
        leverage.push(w * sum(&squares));
    }
    let deviances: Vec<f64> = solution
        .eta
        .iter()
        .enumerate()
        .map(|(row, &eta)| model.deviance(row, eta))
        .collect();
    let deviance = sum(&deviances);
    let fitted = Fitted {
        kept: chosen.kept.clone(),
        omitted: chosen.omitted.clone(),
        zero: chosen.zero.clone(),
        vanishing: chosen.vanishing.clone(),
        oversized: chosen.oversized.clone(),
        estimates: solution.coefficients.clone(),
        r: solution.r.clone(),
        scales: solution.scales.clone(),
        residuals,
        fitted: fitted_values,
        leverage,
        residual_sum_of_squares: deviance,
        condition_number: condition_number(&solution.r),
    };
    let mut notes = Vec::new();
    dropped_notes(ctx, &mut notes, &design, &fitted, &[], "");

    // The null model: the intercept alone, whose maximum is the mean of the outcome, or
    // nothing at all without an intercept.
    let null_eta = if design.intercept {
        let mean = sum(&y) / n as f64;
        if mean > 0.0 && mean < 1.0 {
            link.link(mean)
        } else {
            0.0
        }
    } else {
        0.0
    };
    let null_terms: Vec<(f64, f64)> = (0..n)
        .map(|row| {
            (
                model.at(row, null_eta, false).log_likelihood,
                model.deviance(row, null_eta),
            )
        })
        .collect();
    let null_log_likelihood = sum(&null_terms.iter().map(|t| t.0).collect::<Vec<_>>());
    let null_deviance = sum(&null_terms.iter().map(|t| t.1).collect::<Vec<_>>());
    // A fit that has not converged has no estimates to report (section 5.6 of the Language
    // Reference: non-convergence is an error with the diagnosis); the message gives the change
    // of the log-likelihood at the last iteration.
    if !solution.converged {
        let last = solution.history.len();
        let change = if last >= 2 {
            solution.history[last - 1] - solution.history[last - 2]
        } else {
            0.0
        };
        return Err(fail(
            Diagnostic::error(
                "Z0803",
                format!(
                    "glm: the fit did not converge in {} iterations; the log-likelihood still changed by {} at the last",
                    solution.iterations,
                    change_text(change)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no maximum found")
            .help(format!(
                "a regressor may nearly separate the outcomes; look at `{outcome}` against each regressor, or drop or combine terms"
            )),
        ));
    }

    // The classical variance: the inverse of the expected information, S R⁻¹ R⁻ᵀ S.
    let inverse = inverse_upper(&solution.r).map_err(|_| internal(ctx, "a zero pivot"))?;
    let vcov = Matrix::from_fn(k, k, |i, j| {
        let products: Vec<f64> = (0..k)
            .map(|l| inverse.get(i, l) * inverse.get(j, l))
            .collect();
        sum(&products) * solution.scales[i] * solution.scales[j]
    });
    let formula_text = formula.to_string();
    let setting = Setting {
        method: "glm",
        formula: &formula_text,
        se_line: "classical",
        df: n.saturating_sub(k),
        level,
        n,
        n_note: n_note(&design),
        more_lines: Vec::new(),
        model: formula.clone(),
        levels: design.category_levels(),
        frame_rows: frame.table.rows(),
        effects: Vec::new(),
        absorbed_parameters: 0,
        singletons: 0,
    };
    let likelihood = LikelihoodFit {
        family,
        link: link.name().to_string(),
        deviance,
        null_deviance,
        df_deviance: n.saturating_sub(k),
        df_null: n.saturating_sub(usize::from(design.intercept)),
        iterations: solution.iterations,
        converged: solution.converged,
        method: match control.method {
            Method::Scoring => "Fisher scoring",
            Method::Newton => "Newton",
        }
        .to_string(),
    };
    let mut fit = summary::likelihood_fit(
        &design,
        &fitted,
        &vcov,
        &setting,
        likelihood,
        solution.log_likelihood,
        null_log_likelihood,
    );
    fit.source = Some(FitSource {
        frame: frame.clone(),
        weights: None,
        cluster: None,
    });
    fit.notes.extend(notes);
    Ok(Value::Fit(Rc::new(fit)))
}
