//! `curve` (catalogue row 515; open decision #112; release 0.9.0, step 14): the curves of a
//! classifier, of which this release has one, the ROC curve of a binary fit with the area
//! under it.
//!
//! The scores are the fitted probabilities. At each distinct score t, in falling order, the
//! sensitivity is the share of the ones whose score is at least t, and the specificity the
//! share of the noughts whose score is below t: one point for every distinct threshold, and
//! rows that share a score move together. The area under the curve through those points from
//! (0, 0), by the trapezoid rule, is the Mann–Whitney statistic of the scores, the probability
//! that a one has a higher score than a nought, a tie counted as half. Without weights every
//! count is a whole number and the area is exact to the last division; under weights a row
//! counts as its weight says.

use std::rc::Rc;
use zeff_core::display::{fixed_grouped, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Alternative, Estimate, Fit, FollowUp, Frame, Statistic, TestResult, Value};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::order::WeightKind;
use zeff_results::design::Design;
use zeff_results::likelihood::Link;

/// The ROC curve of scores against 0/1 outcomes.
#[derive(Debug, Clone, PartialEq)]
pub struct Roc {
    /// The distinct scores, in falling order.
    pub thresholds: Vec<f64>,
    /// At each threshold, the share of the ones scored at or above it.
    pub sensitivity: Vec<f64>,
    /// At each threshold, the share of the noughts scored below it.
    pub specificity: Vec<f64>,
    /// The area under the curve.
    pub area: f64,
    /// The ones and the noughts, counted by their weights.
    pub ones: f64,
    pub noughts: f64,
}

/// The ROC curve of `scores` against `outcomes` (1 for a one, anything else a nought), each
/// row counted by its weight when `weights` are given: `None` when the rows hold no one or no
/// nought, for then there is no curve.
pub fn roc(scores: &[f64], outcomes: &[f64], weights: Option<&[f64]>) -> Option<Roc> {
    let weight = |row: usize| weights.map_or(1.0, |w| w[row]);
    // The rows by falling score; rows of one score keep their order, so the sums are taken in
    // one order on every machine.
    let mut order: Vec<usize> = (0..scores.len()).collect();
    order.sort_by(|&a, &b| scores[b].total_cmp(&scores[a]).then(a.cmp(&b)));
    let mut thresholds = Vec::new();
    // The ones and the noughts at or above each threshold, counted by their weights as the
    // rows are passed in falling order of score.
    let (mut ones_above, mut noughts_above) = (Vec::new(), Vec::new());
    // Twice the area times ones × noughts, a trapezoid for each threshold: with whole weights
    // every term is a whole number.
    let mut trapezoids = Vec::new();
    let (mut above_ones, mut above_noughts) = (0.0_f64, 0.0_f64);
    let mut at = 0;
    while at < order.len() {
        let score = scores[order[at]];
        let (mut tied_ones, mut tied_noughts) = (0.0_f64, 0.0_f64);
        while at < order.len() && scores[order[at]] == score {
            let row = order[at];
            if outcomes[row] == 1.0 {
                tied_ones += weight(row);
            } else {
                tied_noughts += weight(row);
            }
            at += 1;
        }
        trapezoids.push(tied_noughts * (2.0 * above_ones + tied_ones));
        above_ones += tied_ones;
        above_noughts += tied_noughts;
        thresholds.push(score);
        ones_above.push(above_ones);
        noughts_above.push(above_noughts);
    }
    // The totals are the counts at the lowest threshold, which every row is at or above, so
    // the curve ends at (1, 1) to the last bit whatever the weights.
    let (ones, noughts) = (above_ones, above_noughts);
    if !(ones > 0.0 && noughts > 0.0) {
        return None;
    }
    let sensitivity: Vec<f64> = ones_above.iter().map(|above| above / ones).collect();
    let specificity: Vec<f64> = noughts_above
        .iter()
        .map(|above| (noughts - above) / noughts)
        .collect();
    let area = zeff_kernels::sum(&trapezoids) / (2.0 * ones * noughts);
    Some(Roc {
        thresholds,
        sensitivity,
        specificity,
        area,
        ones,
        noughts,
    })
}

const NEWDATA: usize = 1;
const TYPE: usize = 2;

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

/// The outcome of the model as its formula writes it: what stands before `~`.
fn outcome_name(fit: &Fit) -> String {
    fit.formula
        .split('~')
        .next()
        .map_or(String::new(), |text| text.trim().to_string())
}

/// The scores, the outcomes and the weights the curve is drawn from, with the rows left out.
struct Rows {
    scores: Vec<f64>,
    outcomes: Vec<f64>,
    weights: Option<Vec<f64>>,
    /// The observations: the rows, or the sum of the frequency weights.
    n: usize,
    dropped: usize,
    /// How the header names where the rows come from.
    of_newdata: bool,
}

/// The rows of `newdata` with nothing missing in a variable of the model: their outcomes and
/// the probabilities the fit predicts for them.
fn new_rows(
    ctx: &mut CallContext<'_>,
    fit: &Fit,
    link: Link,
    newdata: &Frame,
) -> Result<Rows, Fail> {
    // Whatever the model matrix says of the new rows is said in the line of the rows used.
    let said = ctx.messages.len();
    let built = Design::build_with(ctx, newdata, &fit.model, &[], &fit.levels);
    ctx.messages.truncate(said);
    let design = built?;
    if let Some(name) = design
        .names
        .iter()
        .find(|n| !fit.terms.contains(n) && !fit.omitted.contains(n))
    {
        return Err(internal(
            ctx,
            &format!("the model matrix of `newdata` has a column `{name}` the fit does not"),
        ));
    }
    let Some(outcomes) = design.outcome() else {
        return Err(internal(ctx, "a binary fit without an outcome"));
    };
    if let Some(other) = outcomes.iter().find(|y| **y != 0.0 && **y != 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{}` is {} in a row of `newdata`, and the curve needs an outcome of 0 and 1",
                    outcome_name(fit),
                    format_float(*other)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(NEWDATA), "an outcome that is not 0 or 1")
            .help("the outcome of `newdata` is coded as the outcome the model was fitted to"),
        ));
    }
    // A coefficient omitted as collinear counts as nought, as `predict` has it.
    let beta: Vec<f64> = design
        .names
        .iter()
        .map(|name| {
            fit.terms
                .iter()
                .position(|t| t == name)
                .map_or(0.0, |at| fit.estimates[at])
        })
        .collect();
    let mut eta = vec![0.0; design.rows.len()];
    for (column, b) in beta.iter().enumerate() {
        if *b == 0.0 {
            continue;
        }
        for (e, x) in eta.iter_mut().zip(design.column_values(column)) {
            *e += b * x;
        }
    }
    Ok(Rows {
        scores: eta.iter().map(|e| link.mean(*e)).collect(),
        outcomes,
        weights: None,
        n: design.rows.len(),
        dropped: newdata.table.rows() - design.rows.len(),
        of_newdata: true,
    })
}

/// `fit.curve(newdata:, type: roc)`: the ROC curve of a binary fit, its points at every
/// distinct fitted probability and the area under it, on the rows the fit used or on the rows
/// of `newdata:`.
pub fn curve(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = match args.get(0) {
        Some(Value::Fit(fit)) if fit.likelihood.is_some() => Rc::clone(fit),
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`curve` needs the fit of a binary model, but this is {}",
                        other.map_or("nothing".to_string(), Value::type_word)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "not a binary fit")
                .help("fit the model with `glm(y ~ x, family: binomial)`"),
            ));
        }
    };
    let likelihood = fit.likelihood.as_deref().expect("checked above");
    let Some(link) = Link::named(&likelihood.link) else {
        return Err(internal(ctx, "a `glm` fit of a link `curve` does not know"));
    };
    match args.get(TYPE) {
        Some(Value::Word(word)) if word == "roc" => {}
        _ => {
            return Err(fail(
                Diagnostic::error("Z0406", "`curve` needs `type:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`type:` is missing")
                    .help("the ROC curve is `type: roc`"),
            ));
        }
    }
    let rows = match args.get(NEWDATA) {
        Some(Value::Frame(newdata)) => new_rows(ctx, &fit, link, &newdata.clone())?,
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`newdata:` of `curve` must be a Frame, but this is {}",
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(NEWDATA), other.type_word()),
            ));
        }
        None => Rows {
            scores: likelihood.eta.iter().map(|e| link.mean(*e)).collect(),
            outcomes: likelihood.outcome.clone(),
            weights: likelihood.weights.clone(),
            n: fit.n,
            dropped: fit.dropped,
            of_newdata: false,
        },
    };
    let outcome = outcome_name(&fit);
    let Some(curve) = roc(&rows.scores, &rows.outcomes, rows.weights.as_deref()) else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("the rows hold only one value of `{outcome}`, so there is no curve"),
            )
            .in_file(ctx.script)
            .primary(
                ctx.span_of(if rows.of_newdata { NEWDATA } else { 0 }),
                "no row with the other outcome",
            )
            .help("an ROC curve sets the rows with a 1 against the rows with a 0, and needs both"),
        ));
    };
    // The lines under the title: the model, when the title names the fit by its binding; the
    // rows, and how many are ones.
    let mut header = Vec::new();
    if fit.name.is_some() {
        header.push(format!(
            "{} \u{b7} {} \u{b7} {}, {}",
            fit.method, fit.formula, likelihood.family, likelihood.link
        ));
    }
    let observations = fixed_grouped(rows.n as f64, 0);
    let source_weights = fit.source.as_ref().and_then(|s| s.weights.clone());
    let sampled = !rows.of_newdata
        && rows.weights.is_some()
        && fit.weight_kind != Some(WeightKind::Frequency);
    let mut line = if sampled {
        format!(
            "N = {observations}, weighted by `{}`",
            source_weights.as_deref().unwrap_or("the weights")
        )
    } else {
        format!(
            "N = {observations}: {} with `{outcome}` 1 and {} with 0",
            fixed_grouped(curve.ones, 0),
            fixed_grouped(curve.noughts, 0)
        )
    };
    if rows.of_newdata {
        line.push_str(" among the rows of `newdata`");
        if rows.dropped > 0 {
            line.push_str(&format!(
                " ({} left out for NA in a variable of the model)",
                count(rows.dropped as u64)
            ));
        }
    }
    header.push(line);
    let points = curve.thresholds.len();
    let after = vec![format!(
        "the curve has {} {}, one for each distinct fitted probability; `estimates()` gives them and `plot()` draws the curve",
        fixed_grouped(points as f64, 0),
        if points == 1 { "point" } else { "points" }
    )];
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "curve".to_string(),
        title: format!(
            "curve \u{b7} {} \u{b7} ROC",
            fit.name.clone().unwrap_or_else(|| fit.formula.clone())
        ),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: None,
        estimate: Some(Estimate {
            label: "area under the curve".to_string(),
            value: curve.area,
            ci: None,
            decimals: 4,
        }),
        statistic: Statistic {
            name: "AUC".to_string(),
            value: curve.area,
            df: Vec::new(),
        },
        p_value: f64::NAN,
        terms: Vec::new(),
        effect: None,
        level: fit.level,
        alpha: None,
        after,
        notes: Vec::new(),
        n: rows.n,
        dropped: rows.dropped,
        follow_up: Some(Rc::new(FollowUp::Curve {
            kind: "roc".to_string(),
            thresholds: curve.thresholds,
            sensitivity: curve.sensitivity,
            specificity: curve.specificity,
        })),
    })))
}

#[cfg(test)]
mod tests {
    use super::roc;

    #[test]
    fn the_curve_of_a_few_rows_by_hand() {
        // Scores 0.9 (one), 0.8 (nought), 0.8 (one), 0.3 (nought), 0.1 (one).
        let scores = [0.9, 0.8, 0.8, 0.3, 0.1];
        let outcomes = [1.0, 0.0, 1.0, 0.0, 1.0];
        let curve = roc(&scores, &outcomes, None).expect("both outcomes");
        assert_eq!(curve.thresholds, [0.9, 0.8, 0.3, 0.1]);
        assert_eq!(curve.sensitivity, [1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 1.0]);
        assert_eq!(curve.specificity, [1.0, 0.5, 0.0, 0.0]);
        // Of the six pairs of a one and a nought: 0.9 beats both (2), the tied 0.8 beats 0.3
        // and ties 0.8 (1.5), 0.1 beats none: 3.5 of 6.
        assert_eq!(curve.area, 3.5 / 6.0);
        assert_eq!((curve.ones, curve.noughts), (3.0, 2.0));
    }

    #[test]
    fn weights_count_rows_as_copies_do() {
        let scores = [0.9, 0.8, 0.8, 0.3, 0.1];
        let outcomes = [1.0, 0.0, 1.0, 0.0, 1.0];
        let weights = [2.0, 1.0, 3.0, 2.0, 1.0];
        let weighted = roc(&scores, &outcomes, Some(&weights)).expect("both outcomes");
        let (mut s, mut y) = (Vec::new(), Vec::new());
        for row in 0..scores.len() {
            for _ in 0..weights[row] as usize {
                s.push(scores[row]);
                y.push(outcomes[row]);
            }
        }
        assert_eq!(weighted, roc(&s, &y, None).expect("both outcomes"));
    }

    #[test]
    fn one_outcome_alone_has_no_curve() {
        assert!(roc(&[0.2, 0.7], &[1.0, 1.0], None).is_none());
        assert!(roc(&[0.2, 0.7], &[0.0, 0.0], None).is_none());
        assert!(roc(&[], &[], None).is_none());
    }

    #[test]
    fn a_perfect_and_a_useless_score() {
        let outcomes = [0.0, 0.0, 1.0, 1.0];
        assert_eq!(
            roc(&[0.1, 0.2, 0.8, 0.9], &outcomes, None).unwrap().area,
            1.0
        );
        assert_eq!(
            roc(&[0.9, 0.8, 0.2, 0.1], &outcomes, None).unwrap().area,
            0.0
        );
        assert_eq!(
            roc(&[0.5, 0.5, 0.5, 0.5], &outcomes, None).unwrap().area,
            0.5
        );
    }
}
