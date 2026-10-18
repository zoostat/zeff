//! Weights (Language Reference, section 4.1, "Weights", and section 6, `weights:` and
//! `weight_type:`): a row with `NA` weight is dropped and counted; a negative weight is an error
//! listing the rows; a zero weight leaves the row out of the estimate and the observation count,
//! with a note giving how many. Frequency weights are whole numbers (a row counted `w` times);
//! analytic and sampling weights are any non-negative Floats.

use crate::column::{Column, ColumnType, Value};

/// Stata's fweight, aweight and pweight (section 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightType {
    Frequency,
    Analytic,
    Sampling,
}

impl WeightType {
    pub fn as_str(self) -> &'static str {
        match self {
            WeightType::Frequency => "frequency",
            WeightType::Analytic => "analytic",
            WeightType::Sampling => "sampling",
        }
    }
}

/// Why a column cannot be the weights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeightError {
    /// Weights are numbers: a Bool, text, date or opaque column cannot weigh rows.
    NotNumeric(ColumnType),
    /// Negative weights, by 1-based row (the first twenty at most) and in all.
    Negative { rows: Vec<usize>, count: usize },
    /// A frequency weight that is not a whole number, by 1-based row (the first twenty) and in all.
    NotWhole { rows: Vec<usize>, count: usize },
    /// An infinite weight, by 1-based row.
    Infinite { rows: Vec<usize>, count: usize },
}

/// The weights of a table's rows, ready for a reducer.
#[derive(Debug, Clone, PartialEq)]
pub struct Weights {
    kind: WeightType,
    /// One weight per row: the value for a row that counts, `0.0` for a row left out (a zero or
    /// an `NA` weight), so that a reducer can multiply without a branch.
    values: Vec<f64>,
    /// Rows whose weight was `NA`: dropped and counted (a warning).
    missing: usize,
    /// Rows whose weight was zero: left out, with a note.
    zeros: usize,
}

const LISTED: usize = 20;

impl Weights {
    /// The weights from a numeric column of the table's rows.
    pub fn from_column(column: &Column, kind: WeightType) -> Result<Weights, WeightError> {
        if !matches!(column.ty(), ColumnType::Int | ColumnType::Float) {
            return Err(WeightError::NotNumeric(column.ty()));
        }
        let mut values = Vec::with_capacity(column.len());
        let (mut missing, mut zeros) = (0usize, 0usize);
        let (mut negative, mut not_whole, mut infinite) = (Vec::new(), Vec::new(), Vec::new());
        for row in 0..column.len() {
            let w = match column.get(row) {
                Some(Value::Int(i)) => i as f64,
                Some(Value::Float(x)) => x,
                _ => {
                    missing += 1;
                    values.push(0.0);
                    continue;
                }
            };
            if w.is_infinite() {
                infinite.push(row + 1);
            } else if w < 0.0 {
                negative.push(row + 1);
            } else if kind == WeightType::Frequency && w.fract() != 0.0 {
                not_whole.push(row + 1);
            } else if w == 0.0 {
                zeros += 1;
            }
            values.push(if w.is_finite() && w > 0.0 { w } else { 0.0 });
        }
        if !negative.is_empty() {
            let count = negative.len();
            negative.truncate(LISTED);
            return Err(WeightError::Negative {
                rows: negative,
                count,
            });
        }
        if !infinite.is_empty() {
            let count = infinite.len();
            infinite.truncate(LISTED);
            return Err(WeightError::Infinite {
                rows: infinite,
                count,
            });
        }
        if !not_whole.is_empty() {
            let count = not_whole.len();
            not_whole.truncate(LISTED);
            return Err(WeightError::NotWhole {
                rows: not_whole,
                count,
            });
        }
        Ok(Weights {
            kind,
            values,
            missing,
            zeros,
        })
    }

    /// Every row counted once: the weights of an unweighted call.
    pub fn none(len: usize) -> Weights {
        Weights {
            kind: WeightType::Frequency,
            values: vec![1.0; len],
            missing: 0,
            zeros: 0,
        }
    }

    pub fn kind(&self) -> WeightType {
        self.kind
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The weight of row `row`; `0.0` for a row left out.
    pub fn of(&self, row: usize) -> f64 {
        self.values.get(row).copied().unwrap_or(0.0)
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Whether row `row` takes part (a positive weight).
    pub fn counts(&self, row: usize) -> bool {
        self.of(row) > 0.0
    }

    /// Rows dropped for an `NA` weight.
    pub fn missing(&self) -> usize {
        self.missing
    }

    /// Rows left out for a zero weight.
    pub fn zeros(&self) -> usize {
        self.zeros
    }

    /// The rows that take part, in order.
    pub fn rows_counted(&self) -> Vec<usize> {
        (0..self.values.len())
            .filter(|&r| self.values[r] > 0.0)
            .collect()
    }

    /// The sum of the weights of the rows that take part: the observation count under frequency
    /// weights, the normalising total under the others.
    pub fn total(&self) -> f64 {
        zeff_numeric::sum(&self.values)
    }
}
