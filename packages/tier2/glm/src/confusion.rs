//! `confusion` (catalogue row 516; open decision #88; release 0.9.0, step 15): the
//! classification table of a binary fit at a cutoff.
//!
//! A row is predicted 1 when its fitted probability is at least the cutoff. The table sets the
//! observed outcome against the predicted one: four counts of rows, or sums of weights under
//! weights. From them the sensitivity (of the ones, the share predicted 1), the specificity
//! (of the noughts, the share predicted 0), the accuracy (the share predicted as observed) and
//! the two predictive values (of the rows predicted 1, the share that are ones; of the rows
//! predicted 0, the share that are noughts). A share of nothing is `NA`.

use crate::rows::{binary_fit, counted, outcome_name, rows_line, rows_of};
use std::rc::Rc;
use zeff_core::display::{fixed, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Estimate, Statistic, Table, TermTest, TestResult, Value,
};
use zeff_diagnostics::Diagnostic;

/// The four cells of a classification table, each a count of rows by their weights.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cells {
    /// Ones predicted 1.
    pub true_positive: f64,
    /// Ones predicted 0.
    pub false_negative: f64,
    /// Noughts predicted 1.
    pub false_positive: f64,
    /// Noughts predicted 0.
    pub true_negative: f64,
}

/// `a / b`, or `NaN` for a share of nothing.
fn share(a: f64, b: f64) -> f64 {
    if b > 0.0 { a / b } else { f64::NAN }
}

impl Cells {
    pub fn ones(&self) -> f64 {
        self.true_positive + self.false_negative
    }

    pub fn noughts(&self) -> f64 {
        self.false_positive + self.true_negative
    }

    pub fn sensitivity(&self) -> f64 {
        share(self.true_positive, self.ones())
    }

    pub fn specificity(&self) -> f64 {
        share(self.true_negative, self.noughts())
    }

    pub fn accuracy(&self) -> f64 {
        share(
            self.true_positive + self.true_negative,
            self.ones() + self.noughts(),
        )
    }

    pub fn positive_predictive_value(&self) -> f64 {
        share(self.true_positive, self.true_positive + self.false_positive)
    }

    pub fn negative_predictive_value(&self) -> f64 {
        share(self.true_negative, self.true_negative + self.false_negative)
    }
}

/// The classification table of `scores` against `outcomes` (1 for a one, anything else a
/// nought) at `cutoff`: a row is predicted 1 when its score is at least the cutoff, and
/// counts as its weight when `weights` are given.
pub fn classify(scores: &[f64], outcomes: &[f64], weights: Option<&[f64]>, cutoff: f64) -> Cells {
    // The weights of each cell, summed by the kernels' fixed-shape sum.
    let mut cells: [Vec<f64>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for (row, (score, outcome)) in scores.iter().zip(outcomes).enumerate() {
        let weight = weights.map_or(1.0, |w| w[row]);
        let cell = match (*outcome == 1.0, *score >= cutoff) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        };
        cells[cell].push(weight);
    }
    Cells {
        true_positive: zeff_kernels::sum(&cells[0]),
        false_negative: zeff_kernels::sum(&cells[1]),
        false_positive: zeff_kernels::sum(&cells[2]),
        true_negative: zeff_kernels::sum(&cells[3]),
    }
}

const NEWDATA: usize = 1;
const CUTOFF: usize = 2;
/// The decimals of a share.
const DECIMALS: usize = 4;

/// `fit.confusion(newdata:, cutoff: 0.5)`: the classification table of a binary fit at a
/// cutoff, with the sensitivity, the specificity, the accuracy and the predictive values, on
/// the rows the fit used or on the rows of `newdata:`.
pub fn confusion(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (fit, link) = binary_fit(ctx, &args, "confusion")?;
    let likelihood = fit.likelihood.as_deref().expect("a binary fit");
    let cutoff = match args.get(CUTOFF) {
        None | Some(Value::Na) => 0.5,
        Some(Value::Float(x)) => *x,
        Some(Value::Int(i)) => *i as f64,
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`cutoff:` of `confusion` is a probability, but this is {}",
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(CUTOFF), other.type_word()),
            ));
        }
    };
    if !(cutoff > 0.0 && cutoff < 1.0) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`cutoff:` is a probability strictly between 0 and 1, but this is {}",
                    format_float(cutoff)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(CUTOFF), "outside (0, 1)")
            .help("a row is predicted 1 when its fitted probability is at least the cutoff; 0.5 unless given"),
        ));
    }
    let rows = rows_of(ctx, &fit, link, args.get(NEWDATA), NEWDATA, "confusion")?;
    let outcome = outcome_name(&fit);
    let cells = classify(
        &rows.scores,
        &rows.outcomes,
        rows.weights.as_deref(),
        cutoff,
    );
    let mut header = Vec::new();
    if fit.name.is_some() {
        header.push(format!(
            "{} \u{b7} {} \u{b7} {}, {}",
            fit.method, fit.formula, likelihood.family, likelihood.link
        ));
    }
    header.push(rows_line(
        &fit,
        &rows,
        &outcome,
        cells.ones(),
        cells.noughts(),
    ));
    header.push(format!(
        "a row is predicted 1 when its fitted probability is at least {}",
        format_float(cutoff)
    ));
    let predicted_one = cells.true_positive + cells.false_positive;
    let predicted_nought = cells.false_negative + cells.true_negative;
    let table = Table {
        behind: None,
        title: String::new(),
        before: Vec::new(),
        columns: vec![
            (String::new(), Align::Left),
            ("predicted 1".to_string(), Align::Right),
            ("predicted 0".to_string(), Align::Right),
            ("total".to_string(), Align::Right),
        ],
        rows: vec![
            vec![
                format!("{outcome} 1"),
                counted(cells.true_positive),
                counted(cells.false_negative),
                counted(cells.ones()),
            ],
            vec![
                format!("{outcome} 0"),
                counted(cells.false_positive),
                counted(cells.true_negative),
                counted(cells.noughts()),
            ],
            vec![
                "total".to_string(),
                counted(predicted_one),
                counted(predicted_nought),
                counted(cells.ones() + cells.noughts()),
            ],
        ],
        after: Vec::new(),
        more: Vec::new(),
        rules: vec![2],
    };
    // The shares, each a row of the result: read by `estimates()`, or one by its name.
    let measures = [
        ("sensitivity", cells.sensitivity()),
        ("specificity", cells.specificity()),
        ("accuracy", cells.accuracy()),
        (
            "positive predictive value",
            cells.positive_predictive_value(),
        ),
        (
            "negative predictive value",
            cells.negative_predictive_value(),
        ),
    ];
    let shown = |value: f64| {
        if value.is_finite() {
            fixed(value, DECIMALS)
        } else {
            "NA".to_string()
        }
    };
    let after = vec![
        format!(
            "sensitivity {} \u{b7} specificity {} \u{b7} accuracy {}",
            shown(measures[0].1),
            shown(measures[1].1),
            shown(measures[2].1)
        ),
        format!(
            "positive predictive value {} \u{b7} negative predictive value {}",
            shown(measures[3].1),
            shown(measures[4].1)
        ),
    ];
    let terms = measures
        .iter()
        .map(|(name, value)| TermTest {
            term: (*name).to_string(),
            statistic: Statistic {
                name: "share".to_string(),
                value: *value,
                df: Vec::new(),
            },
            p_value: f64::NAN,
            effect: None,
            estimate: Some(Estimate {
                label: (*name).to_string(),
                value: *value,
                ci: None,
                decimals: DECIMALS,
            }),
            std_error: None,
        })
        .collect();
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "confusion".to_string(),
        title: format!(
            "confusion \u{b7} {} \u{b7} cutoff {}",
            fit.name.clone().unwrap_or_else(|| fit.formula.clone()),
            format_float(cutoff)
        ),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: Some(table),
        estimate: None,
        statistic: Statistic {
            name: "accuracy".to_string(),
            value: cells.accuracy(),
            df: Vec::new(),
        },
        p_value: f64::NAN,
        terms,
        effect: None,
        level: fit.level,
        alpha: None,
        after,
        notes: Vec::new(),
        n: rows.n,
        dropped: rows.dropped,
        follow_up: None,
    })))
}

#[cfg(test)]
mod tests {
    use super::classify;

    #[test]
    fn a_table_counted_by_hand() {
        // Scores at, above and below a cutoff of 0.5; a row at the cutoff is predicted 1.
        let scores = [0.9, 0.5, 0.5, 0.49999, 0.2, 0.7];
        let outcomes = [1.0, 1.0, 0.0, 1.0, 0.0, 0.0];
        let cells = classify(&scores, &outcomes, None, 0.5);
        assert_eq!(
            (
                cells.true_positive,
                cells.false_negative,
                cells.false_positive,
                cells.true_negative
            ),
            (2.0, 1.0, 2.0, 1.0)
        );
        assert_eq!(cells.sensitivity(), 2.0 / 3.0);
        assert_eq!(cells.specificity(), 1.0 / 3.0);
        assert_eq!(cells.accuracy(), 0.5);
        assert_eq!(cells.positive_predictive_value(), 0.5);
        assert_eq!(cells.negative_predictive_value(), 0.5);
    }

    #[test]
    fn weights_count_rows_as_copies_do() {
        let scores = [0.9, 0.5, 0.4, 0.2];
        let outcomes = [1.0, 0.0, 1.0, 0.0];
        let weights = [2.0, 3.0, 1.0, 4.0];
        let weighted = classify(&scores, &outcomes, Some(&weights), 0.45);
        let (mut s, mut y) = (Vec::new(), Vec::new());
        for row in 0..scores.len() {
            for _ in 0..weights[row] as usize {
                s.push(scores[row]);
                y.push(outcomes[row]);
            }
        }
        assert_eq!(weighted, classify(&s, &y, None, 0.45));
    }

    #[test]
    fn a_share_of_nothing_is_not_a_number() {
        // Nothing reaches the cutoff: no row is predicted 1.
        let cells = classify(&[0.2, 0.3], &[1.0, 0.0], None, 0.9);
        assert!(cells.positive_predictive_value().is_nan());
        assert_eq!(cells.sensitivity(), 0.0);
        assert_eq!(cells.specificity(), 1.0);
        // No ones among the rows: no sensitivity.
        assert!(classify(&[0.2], &[0.0], None, 0.5).sensitivity().is_nan());
    }
}
