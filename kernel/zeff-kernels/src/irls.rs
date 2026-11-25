//! The shared optimiser of the likelihood models (Engine and Project Specification, section 14,
//! the row of 0.9.0: "the likelihood trait and shared optimiser; IRLS and Newton fitting"):
//! the maximum of a log-likelihood that depends on each row through one linear predictor,
//! η = Xβ, found by iteratively reweighted least squares.
//!
//! Each iteration solves the weighted least-squares problem of the working response
//! z = η + s / w on the columns of X, with s the row's score ∂ℓ/∂η and w its information in η;
//! with the expected information this is Fisher scoring (R's `glm.fit`), with the observed
//! information it is Newton's method, and for a canonical link the two are one. The weighted
//! problem is solved as the estimators of `regress` solve theirs: the tall-skinny QR of
//! [√w X  √w z] in the engine's fixed blocks, the columns of X scaled to unit norm once, and back
//! substitution. A step that lowers the log-likelihood by more than rounding is halved, at most
//! [`Control::max_halvings`] times; the iterations stop when the log-likelihood changes by no
//! more than `tolerance · (|ℓ| + 0.1)`, the rule of R's `glm.fit` on the deviance (which for a
//! binary outcome is −2ℓ), and no coefficient moves by more than `step_tolerance` of its size. Every sum is a reduction of the fixed shape, and the model's own
//! arithmetic (the exponential, the normal distribution) is the model's, passed in through
//! [`IndexModel`], so the bits are the same on every target.

use crate::matrix::Matrix;
use crate::qr::{BLOCK_ROWS, TallSkinnyQr};
use crate::reduce::{LaneSum, dot, sum};
use crate::triangular::solve_upper;

/// What one row gives at a value of its linear predictor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowTerms {
    /// The row's log-likelihood, its prior weight included.
    pub log_likelihood: f64,
    /// ∂ℓ/∂η, the prior weight included.
    pub score: f64,
    /// The information in η that the iteration weights the row by: expected for Fisher
    /// scoring, observed for Newton's method; zero or more, the prior weight included.
    pub weight: f64,
}

/// A model whose log-likelihood is a sum over rows, each depending on the parameters through
/// one linear predictor.
pub trait IndexModel {
    /// The number of rows.
    fn rows(&self) -> usize;
    /// What `row` gives at `eta`; `observed` asks for the observed information in the place of
    /// the expected.
    fn at(&self, row: usize, eta: f64, observed: bool) -> RowTerms;
}

/// Which information weights the rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// The expected information: Fisher scoring, iteratively reweighted least squares.
    Scoring,
    /// The observed information: Newton's method.
    Newton,
}

/// When to stop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Control {
    pub method: Method,
    /// The relative change of the log-likelihood at which the iterations may stop.
    pub tolerance: f64,
    /// The largest relative change of a coefficient (of a column scaled to unit norm) at which
    /// they may stop; both must hold.
    pub step_tolerance: f64,
    pub max_iterations: usize,
    /// How many times a step that lowers the log-likelihood is halved before the fit stops.
    pub max_halvings: usize,
    /// The rows of each block of the QR (the engine's fixed blocks; a test may ask for small
    /// ones, which change no bit of a result only up to the fold's own rounding).
    pub block_rows: usize,
}

impl Default for Control {
    fn default() -> Control {
        Control {
            method: Method::Scoring,
            tolerance: 1e-13,
            step_tolerance: 1e-11,
            max_iterations: 100,
            max_halvings: 30,
            block_rows: BLOCK_ROWS,
        }
    }
}

/// The maximum found.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    /// One coefficient per column of X, in the units of X.
    pub coefficients: Vec<f64>,
    pub log_likelihood: f64,
    /// The log-likelihood at the start and after each accepted iteration.
    pub history: Vec<f64>,
    pub iterations: usize,
    /// How many steps were halved, in all.
    pub halvings: usize,
    /// Whether the fit converged: the change of the log-likelihood within its tolerance and the
    /// step within its own, or, on a nearly singular design, the log-likelihood still and the
    /// steps no longer shrinking, or no point along a step above the present one.
    pub converged: bool,
    /// The R factor of √w X at the solution, with w the information `Control::method` names
    /// and the columns of X scaled by `scales`: k × k, upper triangular, so that the inverse of
    /// the information is S R⁻¹ R⁻ᵀ S.
    pub r: Matrix,
    /// The scale of each column of X: the reciprocal of its norm (one for a zero column).
    pub scales: Vec<f64>,
    /// The linear predictor of each row at the solution.
    pub eta: Vec<f64>,
}

/// Why no maximum was found.
#[derive(Clone, Debug, PartialEq)]
pub enum Stop {
    /// The log-likelihood, a score or a weight was not a finite number at the start, or at a
    /// point no halving could leave.
    NotFinite { iteration: usize },
    /// The weighted columns are linearly dependent: a column with no information left, as when
    /// the weights of every row it touches have vanished.
    RankLost { iteration: usize, column: usize },
}

/// A step at least this share of the one before it has not shrunk: Fisher scoring and Newton's
/// method shrink their steps by far more at every iteration until rounding takes over.
const STALL_RATIO: f64 = 0.5;
/// How many steps in a row may fail to shrink, the log-likelihood standing still within its
/// tolerance at each, before the fit is taken as converged to the precision the design allows.
const STALLED_STEPS: usize = 2;

/// How small a diagonal entry of R may be, relative to the largest, before the weighted columns
/// count as dependent; the columns are scaled to unit norm, so this is a share of a unit.
const RANK_TOLERANCE: f64 = 1e-12;

/// The maximum of the model's log-likelihood over the coefficients of the columns of `x`,
/// starting from the linear predictor `start_eta` (one value per row), as R's `glm.fit` starts
/// from its `mustart`.
pub fn irls(
    x: &Matrix,
    model: &dyn IndexModel,
    start_eta: &[f64],
    control: &Control,
) -> Result<Solution, Stop> {
    let n = x.rows();
    let k = x.cols();
    assert_eq!(model.rows(), n, "the model needs one row per row of x");
    assert_eq!(start_eta.len(), n, "the start needs one value per row of x");
    let observed = control.method == Method::Newton;
    // The columns scaled to unit norm once, as the least-squares estimators scale theirs.
    let scales: Vec<f64> = (0..k)
        .map(|c| {
            let mut lanes = LaneSum::new();
            lanes.push_squares(x.column(c));
            let s = lanes.value();
            if s > 0.0 && s.is_finite() {
                1.0 / s.sqrt()
            } else {
                1.0
            }
        })
        .collect();
    let scaled = Matrix::from_fn(n, k, |i, j| x.get(i, j) * scales[j]);

    let mut eta = start_eta.to_vec();
    let mut terms = evaluate(model, &eta, observed);
    let mut ll = log_likelihood(&terms).ok_or(Stop::NotFinite { iteration: 0 })?;
    let mut history = vec![ll];
    let mut beta: Option<Vec<f64>> = None;
    let mut halvings = 0;
    let mut converged = false;
    let mut iterations = 0;
    // The size of the last step in the linear predictor, and how many steps in a row have
    // failed to shrink while the log-likelihood stood still: the sign that the fit has reached
    // the precision the design allows (see `STALLED_STEPS`).
    let mut previous_moved = f64::INFINITY;
    let mut stalled = 0;
    while iterations < control.max_iterations {
        iterations += 1;
        let (proposal, _) = weighted_step(&scaled, &eta, &terms, iterations)?;
        let mut candidate = proposal;
        let mut halved = 0;
        let accepted = loop {
            let candidate_eta = predictor(&scaled, &candidate);
            let candidate_terms = evaluate(model, &candidate_eta, observed);
            let candidate_ll = log_likelihood(&candidate_terms);
            let worse = match candidate_ll {
                None => true,
                Some(value) => {
                    // The first step leaves a start that no coefficients gave, so it is taken
                    // as it comes, as R's glm.fit takes it; a later one may not lower the
                    // log-likelihood by more than rounding.
                    beta.is_some() && value < ll - control.tolerance * (ll.abs() + 0.1)
                }
            };
            if !worse {
                break Some((
                    candidate_eta,
                    candidate_terms,
                    candidate_ll.expect("a finite log-likelihood"),
                ));
            }
            let Some(previous) = &beta else {
                return Err(Stop::NotFinite {
                    iteration: iterations,
                });
            };
            if halved == control.max_halvings {
                if candidate_ll.is_some() {
                    // Every point along the step, down to a billionth of it, has a finite
                    // log-likelihood that is not above the present one. The step is an ascent
                    // direction wherever the score is not zero, so the score is zero to the
                    // precision the log-likelihood is computed with: the present point is the
                    // maximum, as nearly as this design lets it be found.
                    break None;
                }
                return Err(Stop::NotFinite {
                    iteration: iterations,
                });
            }
            halved += 1;
            candidate = previous
                .iter()
                .zip(&candidate)
                .map(|(b, c)| 0.5 * (b + c))
                .collect();
        };
        halvings += halved;
        let Some((next_eta, next_terms, next_ll)) = accepted else {
            // No step was taken in this iteration.
            iterations -= 1;
            converged = true;
            break;
        };
        let change = (next_ll - ll).abs();
        let bound = control.tolerance * (next_ll.abs() + 0.1);
        // The step in the coefficients, in the scaled columns' units: near the maximum the
        // log-likelihood is flat to second order, so it stops changing while the coefficients
        // still move in their tenth digit, and the iterations go on until both are still.
        let moved = match &beta {
            None => f64::INFINITY,
            Some(previous) => previous
                .iter()
                .zip(&candidate)
                .map(|(b, c)| (c - b).abs() / c.abs().max(1.0))
                .fold(0.0_f64, f64::max),
        };
        let moved_eta = eta
            .iter()
            .zip(&next_eta)
            .map(|(old, new)| (new - old).abs() / new.abs().max(1.0))
            .fold(0.0_f64, f64::max);
        beta = Some(candidate);
        eta = next_eta;
        terms = next_terms;
        ll = next_ll;
        history.push(ll);
        if change <= bound && moved <= control.step_tolerance {
            converged = true;
            break;
        }
        // On a nearly singular design the estimates cannot settle to the step tolerance: along
        // the directions the design barely tells apart they move by rounding alone, for ever,
        // and where two large estimates cancel in the linear predictor it keeps fewer digits
        // too. The step is therefore also measured where the fit is well determined, in the
        // linear predictor: steps there that no longer shrink while the log-likelihood stands
        // still are rounding, and the fit has converged to the precision the design allows.
        if change <= bound && moved_eta >= STALL_RATIO * previous_moved {
            stalled += 1;
        } else {
            stalled = 0;
        }
        if stalled >= STALLED_STEPS {
            converged = true;
            break;
        }
        previous_moved = moved_eta;
    }
    let beta = beta.unwrap_or_else(|| vec![0.0; k]);
    // The R factor of √w X at the solution, for the variance of the estimates.
    let (_, r) = weighted_step(&scaled, &eta, &terms, iterations)?;
    Ok(Solution {
        coefficients: beta.iter().zip(&scales).map(|(b, s)| b * s).collect(),
        log_likelihood: ll,
        history,
        iterations,
        halvings,
        converged,
        r,
        scales,
        eta,
    })
}

/// Each row's terms at the linear predictor.
fn evaluate(model: &dyn IndexModel, eta: &[f64], observed: bool) -> Vec<RowTerms> {
    eta.iter()
        .enumerate()
        .map(|(row, &e)| model.at(row, e, observed))
        .collect()
}

/// The log-likelihood of the rows, if every term is finite.
fn log_likelihood(terms: &[RowTerms]) -> Option<f64> {
    if terms
        .iter()
        .any(|t| !t.log_likelihood.is_finite() || !t.score.is_finite() || !t.weight.is_finite())
    {
        return None;
    }
    let values: Vec<f64> = terms.iter().map(|t| t.log_likelihood).collect();
    let total = sum(&values);
    total.is_finite().then_some(total)
}

/// Xβ, row by row, in the reduction shape.
fn predictor(x: &Matrix, beta: &[f64]) -> Vec<f64> {
    (0..x.rows()).map(|i| dot(&x.row(i), beta)).collect()
}

/// The weighted least-squares step at `eta`: the coefficients of the working response on the
/// scaled columns, and the R factor of √w X.
fn weighted_step(
    x: &Matrix,
    eta: &[f64],
    terms: &[RowTerms],
    iteration: usize,
) -> Result<(Vec<f64>, Matrix), Stop> {
    let n = x.rows();
    let k = x.cols();
    let mut qr = TallSkinnyQr::new(k + 1);
    let mut row = vec![0.0; k + 1];
    for i in 0..n {
        let w = terms[i].weight;
        if w.is_nan() || w <= 0.0 {
            // A row with no information adds nothing to the weighted problem; its score is
            // zero with it, or the information was not finite and the step stopped earlier.
            continue;
        }
        let root = w.sqrt();
        for (j, slot) in row.iter_mut().enumerate().take(k) {
            *slot = root * x.get(i, j);
        }
        // √w z = √w η + s / √w.
        row[k] = root * eta[i] + terms[i].score / root;
        qr.push_row(&row);
    }
    let augmented = qr.finish();
    let r = Matrix::from_fn(k, k, |i, j| augmented.get(i, j));
    let largest = (0..k).fold(0.0_f64, |m, i| m.max(r.get(i, i).abs()));
    if let Some(column) = (0..k).find(|&i| {
        let d = r.get(i, i).abs();
        d.is_nan() || d <= RANK_TOLERANCE * largest
    }) {
        return Err(Stop::RankLost { iteration, column });
    }
    let qty: Vec<f64> = (0..k).map(|i| augmented.get(i, k)).collect();
    let beta = solve_upper(&r, &qty).map_err(|z| Stop::RankLost {
        iteration,
        column: z.index,
    })?;
    Ok((beta, r))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Poisson log-likelihood with the log link, written with Rust's own exponential: a test
    /// of the iteration alone, whose answer for one group is known in closed form.
    struct Poisson<'a> {
        y: &'a [f64],
    }

    impl IndexModel for Poisson<'_> {
        fn rows(&self) -> usize {
            self.y.len()
        }
        fn at(&self, row: usize, eta: f64, _observed: bool) -> RowTerms {
            let mu = eta.exp();
            RowTerms {
                log_likelihood: self.y[row] * eta - mu,
                score: self.y[row] - mu,
                weight: mu,
            }
        }
    }

    #[test]
    fn an_intercept_alone_reaches_the_log_of_the_mean() {
        let y = [0.0, 1.0, 3.0, 2.0, 4.0, 1.0, 0.0, 5.0];
        let x = Matrix::from_fn(y.len(), 1, |_, _| 1.0);
        let model = Poisson { y: &y };
        let solution = irls(&x, &model, &[0.0; 8], &Control::default()).unwrap();
        let mean: f64 = y.iter().sum::<f64>() / y.len() as f64;
        assert!(solution.converged);
        assert!((solution.coefficients[0] - mean.ln()).abs() < 1e-14);
        // The information of the intercept is n·μ: R is √(nμ) in the scaled column's units.
        let information = (solution.r.get(0, 0) / solution.scales[0]).powi(2);
        assert!((information - y.len() as f64 * mean).abs() < 1e-10 * information);
    }

    #[test]
    fn the_log_likelihood_never_falls_and_scoring_equals_newton_for_a_canonical_link() {
        let y = [0.0, 1.0, 3.0, 2.0, 4.0, 1.0, 0.0, 5.0, 2.0, 7.0];
        let x = Matrix::from_fn(10, 2, |i, j| if j == 0 { 1.0 } else { i as f64 / 3.0 });
        let model = Poisson { y: &y };
        let scoring = irls(&x, &model, &[0.5; 10], &Control::default()).unwrap();
        for pair in scoring.history.windows(2).skip(1) {
            assert!(pair[1] >= pair[0] - 1e-12 * pair[0].abs());
        }
        let newton = irls(
            &x,
            &model,
            &[0.5; 10],
            &Control {
                method: Method::Newton,
                ..Control::default()
            },
        )
        .unwrap();
        for (a, b) in scoring.coefficients.iter().zip(&newton.coefficients) {
            assert!((a - b).abs() < 1e-12 * a.abs().max(1.0));
        }
    }

    /// A nearly singular design (release 0.9.0, step 7d): three columns of which the third is
    /// the second but for a part in a hundred million, so that two large estimates cancel and
    /// neither settles to the step tolerance. The fit stops when the steps of the linear
    /// predictor stop shrinking, converged to the precision the design allows: the fitted
    /// means are those of the well-conditioned fit on the two columns that span the same space
    /// but for that part, and the score is zero in every column.
    #[test]
    fn a_nearly_singular_design_converges_to_the_precision_it_allows() {
        let n = 40;
        let a: Vec<f64> = (0..n).map(|i| ((i * 7) % 11) as f64 / 5.0 - 1.0).collect();
        let u: Vec<f64> = (0..n).map(|i| ((i * 5) % 13) as f64 / 6.0 - 1.0).collect();
        let y: Vec<f64> = (0..n).map(|i| ((i * 3) % 5) as f64).collect();
        let near = Matrix::from_fn(n, 3, |i, j| match j {
            0 => 1.0,
            1 => a[i],
            _ => a[i] + 1e-8 * u[i],
        });
        let model = Poisson { y: &y };
        let start = vec![0.5; n];
        let solution = irls(&near, &model, &start, &Control::default()).unwrap();
        assert!(solution.converged, "{} iterations", solution.iterations);
        assert!(
            solution.iterations < 30,
            "{} iterations",
            solution.iterations
        );
        // The same model with the third column written as what tells it from the second.
        let plain = Matrix::from_fn(n, 3, |i, j| match j {
            0 => 1.0,
            1 => a[i],
            _ => u[i],
        });
        let reference = irls(&plain, &model, &start, &Control::default()).unwrap();
        assert!(
            (solution.log_likelihood - reference.log_likelihood).abs()
                <= 1e-9 * reference.log_likelihood.abs()
        );
        for (ours, theirs) in solution.eta.iter().zip(&reference.eta) {
            assert!((ours - theirs).abs() <= 1e-6, "{ours} and {theirs}");
        }
        for j in 0..3 {
            let terms: Vec<f64> = (0..n)
                .map(|i| near.get(i, j) * model.at(i, solution.eta[i], false).score)
                .collect();
            let size: f64 = terms.iter().map(|t| t.abs()).sum();
            let score: f64 = terms.iter().sum();
            assert!(score.abs() <= 1e-6 * size, "column {j}: {score} of {size}");
        }
    }

    #[test]
    fn a_column_with_no_weight_is_found() {
        let y = [1.0, 2.0, 3.0];
        let x = Matrix::from_fn(3, 2, |_, j| if j == 0 { 1.0 } else { 0.0 });
        let model = Poisson { y: &y };
        let stopped = irls(&x, &model, &[0.0; 3], &Control::default()).unwrap_err();
        assert_eq!(
            stopped,
            Stop::RankLost {
                iteration: 1,
                column: 1
            }
        );
    }

    // The tests below are of release 0.9.0, step 42c: the first weekly run of the mutation job
    // on this file (45437619962, 24 November 2026) left eighteen changes of it that the tests
    // above, and every fit of `glm`, let pass.

    /// One row and one column of ones, with the row's terms given point by point: the linear
    /// predictor is the coefficient, and every value the iteration meets is the test's choice.
    struct Scripted<F: Fn(f64) -> (f64, f64, f64)>(F);

    impl<F: Fn(f64) -> (f64, f64, f64)> IndexModel for Scripted<F> {
        fn rows(&self) -> usize {
            1
        }
        fn at(&self, _row: usize, eta: f64, _observed: bool) -> RowTerms {
            let (log_likelihood, score, weight) = (self.0)(eta);
            RowTerms {
                log_likelihood,
                score,
                weight,
            }
        }
    }

    fn scripted(terms: impl Fn(f64) -> (f64, f64, f64)) -> Result<Solution, Stop> {
        let x = Matrix::from_fn(1, 1, |_, _| 1.0);
        irls(&x, &Scripted(terms), &[0.0], &Control::default())
    }

    /// The fall of the log-likelihood that a step may make from a log-likelihood of zero: the
    /// tolerance times a tenth.
    fn allowed_fall() -> f64 {
        Control::default().tolerance * 0.1
    }

    #[test]
    fn a_step_that_lowers_the_log_likelihood_by_no_more_than_rounding_is_taken() {
        // From 0 the first step goes to 1, where the log-likelihood is zero, and the second
        // to 2, where it is lower by half the fall allowed, and then by exactly the fall
        // allowed: both are taken whole, and the fit rests at 2, where the score is zero.
        for fall in [0.5 * allowed_fall(), allowed_fall()] {
            let solution = scripted(|eta| match eta {
                0.0 => (-1.0, 1.0, 1.0),
                1.0 => (0.0, 1.0, 1.0),
                2.0 => (-fall, 0.0, 1.0),
                _ => (-1.0, 0.0, 1.0),
            })
            .unwrap();
            assert_eq!(solution.coefficients, vec![2.0], "a fall of {fall}");
            assert_eq!(solution.halvings, 0);
            assert_eq!(solution.iterations, 3);
            assert!(solution.converged);
            assert_eq!(solution.history, vec![-1.0, 0.0, -fall, -fall]);
        }
    }

    #[test]
    fn a_step_that_lowers_the_log_likelihood_by_more_is_halved() {
        // The step to 2 would lower the log-likelihood by five times the fall allowed: half
        // of it is taken, to 1.5, where the log-likelihood is no lower and the score is zero.
        let solution = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            1.0 => (0.0, 1.0, 1.0),
            2.0 => (-5.0 * allowed_fall(), 0.0, 1.0),
            1.5 => (0.0, 0.0, 1.0),
            _ => (-1.0, 0.0, 1.0),
        })
        .unwrap();
        assert_eq!(solution.coefficients, vec![1.5]);
        assert_eq!(solution.halvings, 1);
        assert_eq!(solution.iterations, 3);
        assert!(solution.converged);
    }

    #[test]
    fn a_step_with_no_point_above_the_present_one_ends_the_fit_where_it_stands() {
        // After the first step every point along the second, down to a billionth of it, has a
        // lower log-likelihood: the present point is the maximum as nearly as it can be
        // found, no step is counted for the iteration that took none, and the thirty halvings
        // are.
        let solution = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            1.0 => (0.0, 1.0, 1.0),
            _ => (-1.0, 0.0, 1.0),
        })
        .unwrap();
        assert_eq!(solution.coefficients, vec![1.0]);
        assert_eq!(solution.iterations, 1);
        assert_eq!(solution.halvings, 30);
        assert!(solution.converged);
        assert_eq!(solution.history, vec![-1.0, 0.0]);
    }

    #[test]
    fn a_step_into_what_is_not_finite_stops_the_fit() {
        let stopped = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            1.0 => (0.0, 1.0, 1.0),
            _ => (f64::NAN, 0.0, 1.0),
        })
        .unwrap_err();
        assert_eq!(stopped, Stop::NotFinite { iteration: 2 });
    }

    #[test]
    fn a_first_step_into_what_is_not_finite_stops_the_fit() {
        // The first step is taken as it comes, but not into a log-likelihood that is not a
        // number: there are no earlier coefficients to halve it towards.
        let stopped = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            _ => (f64::NAN, 0.0, 1.0),
        })
        .unwrap_err();
        assert_eq!(stopped, Stop::NotFinite { iteration: 1 });
    }

    #[test]
    fn no_iterations_leave_coefficients_of_zero() {
        let y = [1.0, 2.0, 3.0];
        let x = Matrix::from_fn(3, 2, |i, j| if j == 0 { 1.0 } else { i as f64 });
        let solution = irls(
            &x,
            &Poisson { y: &y },
            &[0.5; 3],
            &Control {
                max_iterations: 0,
                ..Control::default()
            },
        )
        .unwrap();
        assert_eq!(solution.coefficients, vec![0.0, 0.0]);
        assert_eq!(solution.iterations, 0);
        assert!(!solution.converged);
        assert_eq!(solution.eta, vec![0.5; 3]);
    }

    #[test]
    fn one_term_that_is_not_finite_at_the_start_stops_the_fit() {
        // Each of the three terms alone: the log-likelihood, the score and the weight.
        for terms in [
            (f64::NEG_INFINITY, 1.0, 1.0),
            (-1.0, f64::INFINITY, 1.0),
            (-1.0, f64::NAN, 1.0),
            (-1.0, 1.0, f64::INFINITY),
            (-1.0, 1.0, f64::NAN),
        ] {
            let stopped = scripted(|_| terms).unwrap_err();
            assert_eq!(stopped, Stop::NotFinite { iteration: 0 }, "{terms:?}");
        }
    }

    /// The Poisson model with rows that carry no information: where `dead` holds, the row's
    /// terms are zero at every linear predictor.
    struct WithDeadRows<'a> {
        y: &'a [f64],
        dead: &'a [bool],
    }

    impl IndexModel for WithDeadRows<'_> {
        fn rows(&self) -> usize {
            self.y.len()
        }
        fn at(&self, row: usize, eta: f64, observed: bool) -> RowTerms {
            if self.dead[row] {
                RowTerms {
                    log_likelihood: 0.0,
                    score: 0.0,
                    weight: 0.0,
                }
            } else {
                Poisson { y: self.y }.at(row, eta, observed)
            }
        }
    }

    #[test]
    fn a_row_with_no_weight_adds_nothing_to_the_fit() {
        let y = [0.0, 1.0, 3.0, 2.0, 4.0, 1.0, 0.0, 5.0, 2.0, 7.0];
        let regressor = |i: usize| i as f64 / 3.0;
        let x = Matrix::from_fn(10, 2, |i, j| if j == 0 { 1.0 } else { regressor(i) });
        let dead = [
            false, false, true, false, false, false, false, true, false, false,
        ];
        let with = irls(
            &x,
            &WithDeadRows { y: &y, dead: &dead },
            &[0.5; 10],
            &Control::default(),
        )
        .unwrap();
        // The same fit on the eight rows that count.
        let kept: Vec<usize> = (0..10).filter(|&i| !dead[i]).collect();
        let y_kept: Vec<f64> = kept.iter().map(|&i| y[i]).collect();
        let x_kept = Matrix::from_fn(8, 2, |i, j| if j == 0 { 1.0 } else { regressor(kept[i]) });
        let without = irls(
            &x_kept,
            &Poisson { y: &y_kept },
            &[0.5; 8],
            &Control::default(),
        )
        .unwrap();
        assert!(with.converged && without.converged);
        for (a, b) in with.coefficients.iter().zip(&without.coefficients) {
            assert!((a - b).abs() < 1e-12 * b.abs().max(1.0), "{a} and {b}");
        }
        assert!((with.log_likelihood - without.log_likelihood).abs() < 1e-12);
    }

    #[test]
    fn a_column_within_the_tolerance_of_another_is_found_under_large_weights() {
        // The third column is the second but for a part in 10¹³, below the tolerance of a part
        // in 10¹² at which a column counts as lost. The means are near a million, so the
        // weights are large and the diagonal of R is in the thousands: the tolerance is a
        // share of the largest entry of that diagonal, whatever its size.
        let n = 40;
        let a: Vec<f64> = (0..n).map(|i| ((i * 7) % 11) as f64 / 5.0 - 1.0).collect();
        let u: Vec<f64> = (0..n).map(|i| ((i * 5) % 13) as f64 / 6.0 - 1.0).collect();
        let y: Vec<f64> = (0..n).map(|i| 1e6 + ((i * 3) % 5) as f64).collect();
        let x = Matrix::from_fn(n, 3, |i, j| match j {
            0 => 1.0,
            1 => a[i],
            _ => a[i] + 1e-13 * u[i],
        });
        let start = vec![1e6_f64.ln(); n];
        let stopped = irls(&x, &Poisson { y: &y }, &start, &Control::default()).unwrap_err();
        assert_eq!(
            stopped,
            Stop::RankLost {
                iteration: 1,
                column: 2
            }
        );
    }

    #[test]
    fn the_iterations_stop_when_the_log_likelihood_and_the_coefficient_are_both_still() {
        // The second step is of a part in 10¹², within the step tolerance, and the fit rests
        // after it. Where that step raises the log-likelihood by less than the bound (the
        // tolerance times the log-likelihood's size and a tenth), both are still and two
        // iterations are counted; where it raises it by more, a third is needed.
        for (rise, iterations) in [(0.5e-14, 2), (1e-13, 3)] {
            let solution = scripted(|eta| match eta {
                0.0 => (-1.0, 1.0, 1.0),
                1.0 => (0.0, 1e-12, 1.0),
                e if e > 1.0 && e < 1.5 => (rise, 0.0, 1.0),
                _ => (-1.0, 0.0, 1.0),
            })
            .unwrap();
            assert!(solution.converged);
            assert_eq!(solution.iterations, iterations, "a rise of {rise}");
            assert_eq!(solution.halvings, 0);
        }
    }

    #[test]
    fn a_step_is_measured_as_a_share_of_a_large_coefficient() {
        // At 1,000 a step of five parts in 10⁹ is five parts in 10¹² of the coefficient,
        // within the step tolerance: the fit has converged with that step.
        let solution = scripted(|eta| match eta {
            0.0 => (-1.0, 1000.0, 1.0),
            1000.0 => (0.0, 5e-9, 1.0),
            e if e > 1000.0 && e < 1000.5 => (0.0, 0.0, 1.0),
            _ => (-1.0, 0.0, 1.0),
        })
        .unwrap();
        assert!(solution.converged);
        assert_eq!(solution.iterations, 2);
        assert!((solution.coefficients[0] - (1000.0 + 5e-9)).abs() < 1e-12);
    }

    #[test]
    fn steps_that_do_not_shrink_while_the_log_likelihood_stands_still_end_the_fit() {
        // The log-likelihood is flat and the score sends the fit to and fro between 1 and
        // 1.01, a step of a hundredth each time: after the first of them, two more that are
        // no smaller than half the one before are rounding, and the fit has converged.
        let solution = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            e if e > 1.005 => (0.0, -0.01, 1.0),
            _ => (0.0, 0.01, 1.0),
        })
        .unwrap();
        assert!(solution.converged);
        assert_eq!(solution.iterations, 4);
        assert!((solution.coefficients[0] - 1.01).abs() < 1e-12);
        // To and fro between 1 and 100: each step is measured as a share of where it lands,
        // 0.99 on the way up and 99 on the way down, and neither is under half the other.
        let solution = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            e if e > 50.0 => (0.0, -99.0, 1.0),
            _ => (0.0, 99.0, 1.0),
        })
        .unwrap();
        assert!(solution.converged);
        assert_eq!(solution.iterations, 3);
        assert_eq!(solution.coefficients, vec![1.0]);
    }

    #[test]
    fn steps_that_shrink_are_followed_to_the_step_tolerance() {
        // The log-likelihood is flat again, but each step covers three quarters of what is
        // left to 2, so each is a quarter of the one before: none is rounding, and the
        // iterations go on until a step is within the step tolerance.
        let solution = scripted(|eta| match eta {
            0.0 => (-1.0, 1.0, 1.0),
            e => (0.0, 0.75 * (2.0 - e), 1.0),
        })
        .unwrap();
        assert!(solution.converged);
        assert!((solution.coefficients[0] - 2.0).abs() < 1e-11);
        assert_eq!(solution.iterations, 20);
    }

    /// Least squares as a likelihood, with an observed information that is twice the expected
    /// one (no model has that; the test is of which of the two the iteration asks for).
    struct Flagged<'a> {
        y: &'a [f64],
    }

    impl IndexModel for Flagged<'_> {
        fn rows(&self) -> usize {
            self.y.len()
        }
        fn at(&self, row: usize, eta: f64, observed: bool) -> RowTerms {
            let residual = self.y[row] - eta;
            RowTerms {
                log_likelihood: -0.5 * residual * residual,
                score: residual,
                weight: if observed { 2.0 } else { 1.0 },
            }
        }
    }

    #[test]
    fn scoring_asks_for_the_expected_information_and_newton_for_the_observed() {
        let y = [1.0, 3.0, 2.0, 5.0, 4.0, 7.0];
        let x = Matrix::from_fn(6, 2, |i, j| if j == 0 { 1.0 } else { (i * i) as f64 });
        let model = Flagged { y: &y };
        let scoring = irls(&x, &model, &[0.0; 6], &Control::default()).unwrap();
        // The columns are brought to unit norm: the scales are one over each column's norm.
        let norms = [
            6.0_f64.sqrt(),
            (1.0_f64 + 16.0 + 81.0 + 256.0 + 625.0).sqrt(),
        ];
        for (scale, norm) in scoring.scales.iter().zip(norms) {
            assert!((scale * norm - 1.0).abs() < 1e-15, "{scale} and {norm}");
        }
        // With the expected information the first step is the least-squares fit, and the
        // second finds nothing to move.
        assert!(scoring.converged);
        assert_eq!(scoring.iterations, 2);
        // R is of the scaled columns with weights of one: its first entry is the norm of a
        // column of unit norm.
        assert!((scoring.r.get(0, 0).abs() - 1.0).abs() < 1e-14);
        let newton = irls(
            &x,
            &model,
            &[0.0; 6],
            &Control {
                method: Method::Newton,
                ..Control::default()
            },
        )
        .unwrap();
        // With an information twice too large each step goes half of the way.
        assert!(newton.converged);
        assert!(newton.iterations > 30, "{}", newton.iterations);
        assert!((newton.r.get(0, 0).abs() - 2.0_f64.sqrt()).abs() < 1e-14);
        for (a, b) in scoring.coefficients.iter().zip(&newton.coefficients) {
            assert!((a - b).abs() < 1e-9 * a.abs().max(1.0), "{a} and {b}");
        }
    }

    #[test]
    fn the_iterations_end_at_their_cap() {
        let y = [1.0, 3.0, 2.0, 5.0, 4.0, 7.0];
        let x = Matrix::from_fn(6, 2, |i, j| if j == 0 { 1.0 } else { (i * i) as f64 });
        let solution = irls(
            &x,
            &Flagged { y: &y },
            &[0.0; 6],
            &Control {
                method: Method::Newton,
                max_iterations: 3,
                ..Control::default()
            },
        )
        .unwrap();
        assert!(!solution.converged);
        assert_eq!(solution.iterations, 3);
        assert_eq!(solution.history.len(), 4);
    }
}
