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
use zeff_kernels::separation::{Separation, separation};
use zeff_kernels::svd::condition_number;
use zeff_kernels::triangular::{inverse_upper, solve_upper_transposed};
use zeff_kernels::{LaneSum, Matrix, sum};
use zeff_numeric::order::WeightKind;
use zeff_results::design::Design;
use zeff_results::estimator::Fitted;
use zeff_results::likelihood::{AtMaximum, Binomial, Likelihood, Link, self_check};
use zeff_results::options::{
    At, CONDITION_WARNING, Choices, checked_variance, choices, cluster_and_weight_warnings,
    clusters_of, observations, se_line, weights_line,
};
use zeff_results::summary::{self, Setting, dropped_notes};
use zeff_results::variance::{Kind, variance};

/// The places of the arguments, as `functions()` declares them.
const FAMILY: usize = 2;
const LINK: usize = 3;
const FE: usize = 4;
const SE: usize = 5;
const CLUSTER: usize = 6;
const WEIGHTS: usize = 7;
const WEIGHT_TYPE: usize = 8;
const LEVEL: usize = 9;

/// The options `glm` shares with `reg`, among its arguments.
const GLM_AT: At = At {
    se: SE,
    cluster: CLUSTER,
    weights: WEIGHTS,
    weight_type: WEIGHT_TYPE,
    level: LEVEL,
};

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

/// What the optimiser's stop says in a message: its text, the mark's label and the help. Since
/// separation is found before the fit and a fit that stops improving is taken as converged, a
/// stop is one of two rare things: a regressor that the weighted design can no longer tell from
/// the others (its rows' probabilities at 0 or 1, or a combination too nearly exact), and a
/// log-likelihood that is not a finite number.
pub fn stop_words(
    stop: &Stop,
    name_of: impl Fn(usize) -> String,
    outcome: &str,
) -> (String, &'static str, String) {
    match stop {
        Stop::RankLost { iteration, column } => {
            let name = name_of(*column);
            (
                format!(
                    "glm: `{name}` cannot be told from the other regressors at iteration {iteration}, so its estimate does not exist"
                ),
                "a regressor with no information left",
                format!(
                    "`{name}` is all but a combination of the regressors before it, or the rows that tell it apart have fitted probabilities at 0 or 1; drop it, or combine it with them"
                ),
            )
        }
        Stop::NotFinite { iteration } => (
            format!("glm: the log-likelihood is not a finite number at iteration {iteration}"),
            "the fit left the range of the arithmetic",
            format!(
                "a regressor with very large values can do this: rescale it, `add(x_k = x / 1000)`, and check how `{outcome}` was coded"
            ),
        ),
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
    // The options shared with `reg`, read and checked as `reg` checks them.
    // Fixed effects in a model fitted by maximum likelihood are their own estimator (the
    // Poisson with absorbed effects of release 1.7); until then `fe:` is refused by name.
    if args.get(FE).is_some() {
        return Err(fail(
            Diagnostic::error("Z0304", "`fe:` on `glm` is not available in this release")
                .in_file(ctx.script)
                .primary(ctx.span_of(FE), "arrives with release 1.7")
                .help("a category among the regressors estimates one coefficient for each of its levels: `y ~ x + industry`; `reg` absorbs fixed effects in a linear model"),
        ));
    }
    let Choices {
        cluster,
        weights,
        weight_kind,
        kind,
        level,
    } = choices(ctx, &args, &GLM_AT, 0)?;
    // Analytic weights say how precisely each row's outcome was measured, which scales the
    // error variance of a linear model; a binary outcome has no error variance to scale (the
    // mean fixes it), so the kind has no meaning here (open decision #199).
    if weights.is_some() && weight_kind == WeightKind::Analytic {
        return Err(fail(
            Diagnostic::error(
                "Z0410",
                format!("`weight_type: analytic` and `family: {family}` exclude each other: the variance of a binary outcome is fixed by its mean, so there is no precision to weight"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "precision weights on a binary outcome")
            .help("`weight_type: frequency` for a row that stands for several identical observations; `weight_type: sampling`, the default, for survey weights"),
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

    // The weight and the cluster variables are given to the design, so that a row where either
    // is `NA` is left out; the weights are then checked and kept on it, as for `reg`.
    let extra: Vec<&str> = weights
        .iter()
        .chain(cluster.iter())
        .map(String::as_str)
        .collect();
    let mut design = Design::build(ctx, &frame, &formula, &extra)?;
    if let Some(name) = &weights {
        design = design.weighted(ctx, &frame, name, weight_kind)?;
    }
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
    let mut notes = Vec::new();
    if design.zero_weights > 0 {
        let text = format!(
            "{} with a zero weight left out of the estimate and the observation count",
            rows_text(design.zero_weights)
        );
        notes.push(text.clone());
        ctx.message(Diagnostic::plain_note(text));
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
    // The rows of X as they are: a likelihood fit weights each row's log-likelihood, not its
    // row of X.
    let full = design.unweighted();
    let x = Matrix::from_fn(n, chosen.kept.len(), |i, j| full.get(i, chosen.kept[j]));
    // Separation, found before any iteration by the linear program of the kernels: a direction
    // along which the likelihood rises for ever names the regressors whose estimates diverge.
    // Under complete separation many directions separate, and the program's is one of them; the
    // regressors named are those that separate the outcome alone (with the intercept), or else,
    // when only a combination does, those the program's direction uses.
    if let Separation::Separated { diverging, .. } = separation(&x, &y) {
        let name_of = |j: usize| design.names[chosen.kept[j]].clone();
        let intercept = (0..x.cols()).find(|&j| name_of(j) == "(intercept)");
        let alone: Vec<usize> = (0..x.cols())
            .filter(|&j| Some(j) != intercept)
            .filter(|&j| {
                let columns: Vec<usize> = intercept.into_iter().chain([j]).collect();
                let sub = Matrix::from_fn(n, columns.len(), |i, c| x.get(i, columns[c]));
                matches!(separation(&sub, &y), Separation::Separated { .. })
            })
            .collect();
        let together = alone.is_empty();
        let named: Vec<usize> = if together {
            diverging
                .into_iter()
                .filter(|&j| Some(j) != intercept)
                .collect()
        } else {
            alone
        };
        let quoted: Vec<String> = named.iter().map(|&j| format!("`{}`", name_of(j))).collect();
        let list = match quoted.len() {
            0 => String::new(),
            1 => quoted[0].clone(),
            m => format!("{} and {}", quoted[..m - 1].join(", "), quoted[m - 1]),
        };
        let (message, help) = match (quoted.len(), together) {
            (0, _) => (
                format!("glm: the regressors together perfectly predict `{outcome}` (separation)"),
                format!("drop or combine the predictors, or check how `{outcome}` was coded"),
            ),
            (1, _) => (
                format!("glm: {list} perfectly predicts `{outcome}` (separation)"),
                format!("drop or combine the predictor, or check how `{outcome}` was coded"),
            ),
            (_, true) => (
                format!("glm: {list} together perfectly predict `{outcome}` (separation)"),
                format!("drop or combine the predictors, or check how `{outcome}` was coded"),
            ),
            (_, false) => (
                format!("glm: {list} each perfectly predict `{outcome}` (separation)"),
                format!("drop or combine the predictors, or check how `{outcome}` was coded"),
            ),
        };
        return Err(fail(
            Diagnostic::error("Z0803", message)
                .in_file(ctx.script)
                .primary(
                    ctx.callee_span,
                    "estimates diverge; maximum likelihood does not exist",
                )
                .help(help),
        ));
    }
    let model = match &design.weights {
        Some(w) => Binomial::weighted(y.clone(), w.clone(), link),
        None => Binomial::new(y.clone(), link),
    };
    let start: Vec<f64> = (0..n).map(|row| model.start(row)).collect();
    let solution = irls(&x, &model, &start, control).map_err(|stop| {
        let (message, label, help) = stop_words(
            &stop,
            |column| design.names[chosen.kept[column]].clone(),
            &outcome,
        );
        fail(
            Diagnostic::error("Z0803", message)
                .in_file(ctx.script)
                .primary(ctx.callee_span, label)
                .help(help),
        )
    })?;

    // The fit in the form every result reads: the estimates, the R factor of the information
    // and its scales, the fitted probabilities, the response residuals and each row's leverage
    // in the weighted fit, h = w xᵀ(XᵀWX)⁻¹x.
    let k = chosen.kept.len();
    let fitted_values: Vec<f64> = solution.eta.iter().map(|&eta| model.mean(eta)).collect();
    let residuals: Vec<f64> = y.iter().zip(&fitted_values).map(|(a, b)| a - b).collect();
    // R's rule for a fitted probability that is 0 or 1 to the machine's precision: within ten
    // machine epsilons of either.
    let edge = 10.0 * f64::EPSILON;
    let at_the_edge = fitted_values
        .iter()
        .filter(|&&mu| mu < edge || mu > 1.0 - edge)
        .count();
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
    dropped_notes(ctx, &mut notes, &design, &fitted, &[], "");

    // The null model: the intercept alone, whose maximum is the mean of the outcome, or
    // nothing at all without an intercept.
    let null_eta = if design.intercept {
        // The weighted mean of the outcome, each row counted by its weight.
        let mean = match &design.weights {
            Some(w) => {
                let products: Vec<f64> = y.iter().zip(w).map(|(a, b)| a * b).collect();
                sum(&products) / sum(w)
            }
            None => sum(&y) / n as f64,
        };
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
    let classical = Matrix::from_fn(k, k, |i, j| {
        let products: Vec<f64> = (0..k)
            .map(|l| inverse.get(i, l) * inverse.get(j, l))
            .collect();
        sum(&products) * solution.scales[i] * solution.scales[j]
    });
    // The self-checks of section 11.1: a failure is a bug in the engine, not a result.
    self_check(&x, &model, &solution, &classical, fitted.condition_number)
        .map_err(|what| internal(ctx, &what))?;
    // The robust and the clustered variance, from the variance layer's one computation: the
    // sandwich of the likelihood's scores between the inverse information, with the factors
    // every model has (HC1 N/(N − k); HC2 and HC3 by the leverage of the weighted fit; CR1 over
    // the clusters).
    let clusters = match &cluster {
        Some(name) => Some(clusters_of(ctx, &frame, name, &design)?),
        None => None,
    };
    let vcov = if kind == Kind::Classical {
        classical
    } else {
        let at_maximum = AtMaximum {
            model: &model,
            x: &x,
            eta: &solution.eta,
            bread: &classical,
        };
        checked_variance(
            ctx,
            variance(
                kind,
                &at_maximum,
                &design,
                &fitted,
                &classical,
                clusters.as_ref(),
                0,
            ),
            kind,
            cluster.as_deref(),
            CLUSTER,
        )?
        .0
    };
    // The statistical warnings, printed here and kept on the fit for the footer of a `report`
    // table: near-singular regressors (W1001), few clusters (W1002), extreme weights (W1003),
    // and probabilities at 0 or 1, where the maximum exists but a regressor all but decides
    // the outcome (W1008).
    let mut warnings: Vec<String> = Vec::new();
    if fitted.condition_number > CONDITION_WARNING {
        let text = format!(
            "near-singular regressors: the condition number of the scaled, weighted design is {:.3e}",
            fitted.condition_number
        );
        warnings.push(format!("warning[W1001]: {text}"));
        ctx.message(Diagnostic::warning("W1001", text).help(
            "two regressors are almost the same column, so their estimates are poorly determined; drop one, or combine them",
        ));
    }
    cluster_and_weight_warnings(ctx, &mut notes, &mut warnings, clusters.as_ref(), &design);
    if at_the_edge > 0 {
        let (text, rows) = if at_the_edge == 1 {
            (
                "1 fitted probability is numerically 0 or 1".to_string(),
                "this row",
            )
        } else {
            (
                format!(
                    "{} fitted probabilities are numerically 0 or 1",
                    count(at_the_edge as u64)
                ),
                "these rows",
            )
        };
        warnings.push(format!("warning[W1008]: {text}"));
        ctx.message(Diagnostic::warning("W1008", text).help(format!(
            "in {rows} the regressors all but decide `{outcome}`; the estimates exist, but look at the largest and smallest values of each regressor"
        )));
    }
    let formula_text = formula.to_string();
    let line = se_line(kind, clusters.as_ref(), cluster.as_deref());
    // The observations: the rows used, or under frequency weights the sum of the weights.
    let (n_obs, note) = observations(&design, None);
    let setting = Setting {
        method: "glm",
        formula: &formula_text,
        se_line: &line,
        df: n_obs.saturating_sub(k),
        level,
        n: n_obs,
        n_note: note,
        more_lines: weights_line(weights.as_deref(), &design),
        model: formula.clone(),
        levels: design.category_levels(),
        frame_rows: frame.table.rows(),
        effects: Vec::new(),
        absorbed_parameters: 0,
        singletons: 0,
    };
    let likelihood = LikelihoodFit {
        outcome: y.clone(),
        eta: solution.eta.clone(),
        weights: design.weights.clone(),
        null_log_likelihood,
        pseudo: weights.is_some() && weight_kind == WeightKind::Sampling,
        family,
        link: link.name().to_string(),
        deviance,
        null_deviance,
        df_deviance: n_obs.saturating_sub(k),
        df_null: n_obs.saturating_sub(usize::from(design.intercept)),
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
        weights: weights.clone(),
        cluster: cluster.clone(),
    });
    fit.notes.extend(notes);
    fit.warnings.extend(warnings);
    Ok(Value::Fit(Rc::new(fit)))
}
