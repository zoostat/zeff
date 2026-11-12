//! `case` (Language Reference, section 5.1), and `case` row by row (section 3.7, "Columns and
//! scalars"): which arm gives the value, for one value and for every row of a column.
//!
//! The arms are taken in order and the first true one wins; its value alone is evaluated. The
//! first condition that is a column turns the rest row-wise: each later arm's condition is
//! evaluated over every row and its value over every row it supplies, the first true arm
//! winning in each row. A row no arm matches is `NA`, and is counted.
//!
//! The kernel asks for the parts of the arms it needs, so that whoever holds the arms
//! evaluates them: the interpreter, for a `case` in a script, and the `plot` package, for one
//! kept in a mapping (open decision #107, option B). Both get the same values, the same
//! warning and the same refusals.

use zeff_columns::ColumnType;
use zeff_diagnostics::{Diagnostic, Span, count};

use crate::lift;
use crate::value::Value;

/// A part of an arm, by the arm's place counted from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Condition(usize),
    Value(usize),
    /// The whole arm: where a refusal that is of no one part points.
    Arm(usize),
}

/// Why a `case` gives no value.
#[derive(Debug, Clone, PartialEq)]
pub enum Trouble {
    /// A condition that is not a Bool, nor a column of them: what it came to.
    Condition { arm: usize, value: Value },
    /// A condition or a value that is a column of another length than the rows the `case`
    /// runs over.
    Rows {
        part: Part,
        found: usize,
        rows: usize,
    },
    /// A value that is a Category column, which does not combine row by row.
    Category { arm: usize },
    /// A value that is a column the engine carries and does not compute on.
    Opaque { arm: usize, name: &'static str },
    /// Values of more than one type across the rows; `start` is the first arm evaluated row
    /// by row.
    Mixed { start: usize },
}

/// A `case` that gave no value: for a reason of its own, or because a part of an arm could
/// not be evaluated.
#[derive(Debug)]
pub enum Failed<E> {
    /// Boxed, so that the refusal weighs little on the way of a `case` that has none.
    Trouble(Box<Trouble>),
    Other(E),
}

/// What a `case` came to.
#[derive(Debug, Clone, PartialEq)]
pub struct Chosen {
    pub value: Value,
    /// The rows that matched no arm, which are `NA`; for a `case` over single values, 1 when
    /// no arm matched.
    pub unmatched: usize,
    /// Whether a condition was a column, so that the `case` ran row by row.
    pub by_rows: bool,
}

impl Chosen {
    /// W0208 for the rows no arm matched, `None` when every row had its arm.
    pub fn warning(&self) -> Option<Diagnostic> {
        if self.unmatched == 0 {
            return None;
        }
        let message = if self.by_rows {
            format!(
                "{} matched no arm of the `case` and {} NA",
                plural_rows(self.unmatched),
                if self.unmatched == 1 { "is" } else { "are" }
            )
        } else {
            "no arm of the `case` matched, so its value is NA".to_string()
        };
        Some(
            Diagnostic::warning("W0208", message).help("add a last arm for the rest, `_ => value`"),
        )
    }
}

/// The value of a `case`. `conditions` says of each arm whether it has a condition, the
/// catch-all `_` having none, and `part` evaluates the condition or the value of an arm when
/// the kernel asks for it; a value is asked for only when its arm supplies a row.
pub fn choose<E>(
    conditions: &[bool],
    mut part: impl FnMut(Part) -> Result<Value, E>,
) -> Result<Chosen, Failed<E>> {
    let one = |value: Value| Chosen {
        value,
        unmatched: 0,
        by_rows: false,
    };
    for (k, has_condition) in conditions.iter().enumerate() {
        if !has_condition {
            return Ok(one(part(Part::Value(k)).map_err(Failed::Other)?));
        }
        match part(Part::Condition(k)).map_err(Failed::Other)? {
            Value::Bool(true) => return Ok(one(part(Part::Value(k)).map_err(Failed::Other)?)),
            Value::Bool(false) | Value::Na => {}
            Value::Column(column) => return by_rows(conditions, k, column, part),
            other => {
                return Err(Failed::Trouble(Box::new(Trouble::Condition {
                    arm: k,
                    value: other,
                })));
            }
        }
    }
    Ok(Chosen {
        value: Value::Na,
        unmatched: 1,
        by_rows: false,
    })
}

/// `case` row by row, from the arm `start` whose condition, `first`, is a column.
fn by_rows<E>(
    conditions: &[bool],
    start: usize,
    first: zeff_columns::Column,
    mut part: impl FnMut(Part) -> Result<Value, E>,
) -> Result<Chosen, Failed<E>> {
    let trouble = |trouble: Trouble| Err(Failed::Trouble(Box::new(trouble)));
    let rows = first.len();
    let mut chosen: Vec<Option<usize>> = vec![None; rows];
    let mut remaining = rows;
    let mut values: Vec<Option<Value>> = vec![None; conditions.len()];
    let mut first = Some(first);
    for (k, has_condition) in conditions.iter().enumerate().skip(start) {
        if remaining == 0 {
            break;
        }
        let covers: Vec<usize> = if !has_condition {
            (0..rows).filter(|r| chosen[*r].is_none()).collect()
        } else {
            let value = match first.take() {
                Some(column) => Value::Column(column),
                None => part(Part::Condition(k)).map_err(Failed::Other)?,
            };
            match value {
                Value::Bool(true) => (0..rows).filter(|r| chosen[*r].is_none()).collect(),
                Value::Bool(false) | Value::Na => Vec::new(),
                Value::Column(column) => {
                    let Some(bools) = column.as_bools() else {
                        return trouble(Trouble::Condition {
                            arm: k,
                            value: Value::Column(column),
                        });
                    };
                    if column.len() != rows {
                        return trouble(Trouble::Rows {
                            part: Part::Condition(k),
                            found: column.len(),
                            rows,
                        });
                    }
                    bools
                        .enumerate()
                        .filter(|(r, b)| *b == Some(true) && chosen[*r].is_none())
                        .map(|(r, _)| r)
                        .collect()
                }
                other => {
                    return trouble(Trouble::Condition {
                        arm: k,
                        value: other,
                    });
                }
            }
        };
        if covers.is_empty() {
            continue;
        }
        let value = part(Part::Value(k)).map_err(Failed::Other)?;
        if let Value::Column(column) = &value
            && column.len() != rows
        {
            return trouble(Trouble::Rows {
                part: Part::Value(k),
                found: column.len(),
                rows,
            });
        }
        remaining -= covers.len();
        for r in covers {
            chosen[r] = Some(k);
        }
        values[k] = Some(value);
    }
    // The values of a `case` are the computable scalar types (section 4.3): a Category is
    // refused rather than turned into its labels unasked (section 3.7, "No implicit
    // conversion"), and an opaque column is not computed on.
    let mut fallback: Option<ColumnType> = None;
    for (k, value) in values.iter().enumerate() {
        if let Some(Value::Column(column)) = value {
            let ty = column.ty();
            match ty {
                ColumnType::Bool
                | ColumnType::Int
                | ColumnType::Float
                | ColumnType::Str
                | ColumnType::Date
                | ColumnType::DateTime => {}
                ColumnType::Category => return trouble(Trouble::Category { arm: k }),
                other => {
                    return trouble(Trouble::Opaque {
                        arm: k,
                        name: other.name(),
                    });
                }
            }
            fallback.get_or_insert(ty);
        }
    }
    let per_row: Vec<Value> = (0..rows)
        .map(|r| match chosen[r] {
            None => Value::Na,
            Some(k) => match &values[k] {
                Some(Value::Column(column)) => lift::cell_value(column, r),
                Some(scalar) => scalar.clone(),
                None => Value::Na,
            },
        })
        .collect();
    let value = match lift::collect_column(&per_row) {
        Some(column) => Value::Column(column),
        None if per_row.iter().all(|v| matches!(v, Value::Na)) => match fallback {
            Some(ty) => Value::Column(zeff_columns::Column::missing(ty, rows)),
            None => Value::Na,
        },
        None => return trouble(Trouble::Mixed { start }),
    };
    Ok(Chosen {
        value,
        unmatched: remaining,
        by_rows: true,
    })
}

/// The refusal of a `case`, in the words of sections 3.7 and 5.1. `at` gives where a part of
/// an arm is written, and `text` a condition as the script wrote it.
pub fn diagnostic(
    trouble: &Trouble,
    script: &str,
    at: impl Fn(Part) -> Span,
    text: impl Fn(usize) -> String,
) -> Diagnostic {
    match trouble {
        Trouble::Condition { arm, value } => {
            let text = text(*arm);
            let (message, label, help) = match value {
                Value::Column(column) => (
                    format!(
                        "a `case` arm's condition must be a Bool or a Bool column, but `{text}` is {} column",
                        with_article(column.ty().name())
                    ),
                    "not a condition",
                    "write a comparison, `x > 0`, or a Bool variable; the catch-all is `_`",
                ),
                other => (
                    format!(
                        "a `case` arm needs one Bool, but `{text}` is {}",
                        other.type_word()
                    ),
                    "not a Bool",
                    "write a comparison, `wages.n_rows() < 30`, or a binding that holds a Bool",
                ),
            };
            Diagnostic::error("Z0203", message)
                .in_file(script)
                .primary(at(Part::Condition(*arm)), label)
                .help(help)
        }
        Trouble::Rows { part, found, rows } => Diagnostic::error(
            "Z0217",
            format!(
                "this `case` runs over {}, but this arm gives {}",
                plural_rows(*rows),
                plural_rows(*found)
            ),
        )
        .in_file(script)
        .primary(at(*part), "a column of another length")
        .help("two columns combine only when they belong to the same frame or have the same length; `merge` joins frames first"),
        Trouble::Category { arm } => Diagnostic::error(
            "Z0214",
            "a `case` arm gives a Category column, which does not combine row by row with the other arms",
        )
        .in_file(script)
        .primary(at(Part::Value(*arm)), "a Category")
        .help("write `to(x, Str)` to use its labels, or `to(x, Int)` for its level positions (section 3.7, \"Conversions\")"),
        Trouble::Opaque { arm, name } => Diagnostic::error(
            "Z0304",
            format!("a `case` giving a {name} column is not available in this release"),
        )
        .in_file(script)
        .primary(at(Part::Value(*arm)), "an opaque column")
        .help("an opaque column is carried, not computed on, until the package that brings its functions (section 4.3 of the Language Reference)"),
        Trouble::Mixed { start } => Diagnostic::error(
            "Z0214",
            "the arms of a `case` give values of more than one type across the rows",
        )
        .in_file(script)
        .primary(at(Part::Arm(*start)), "the first arm evaluated row by row")
        .help("every arm must give one type: Int and Float join as Float, NA fits any type, and text needs `to(x, Str)`"),
    }
}

fn with_article(name: &str) -> String {
    let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {name}")
}

fn plural_rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeff_columns::Column;

    /// A `case` over parts given outright; the parts asked for are recorded.
    fn run(
        arms: &[(Option<Value>, Value)],
        asked: &mut Vec<Part>,
    ) -> Result<Chosen, Failed<String>> {
        let conditions: Vec<bool> = arms.iter().map(|(c, _)| c.is_some()).collect();
        choose(&conditions, |part| {
            asked.push(part);
            match part {
                Part::Condition(k) => arms[k].0.clone().ok_or("no condition".to_string()),
                Part::Value(k) => Ok(arms[k].1.clone()),
                Part::Arm(_) => Err("an arm is not a part to evaluate".to_string()),
            }
        })
    }

    fn bools(values: &[Option<bool>]) -> Value {
        Value::Column(Column::bools(values.iter().copied()))
    }

    #[test]
    fn the_first_true_arm_wins_and_its_value_alone_is_evaluated() {
        let mut asked = Vec::new();
        let chosen = run(
            &[
                (Some(Value::Bool(false)), Value::Int(1)),
                (Some(Value::Na), Value::Int(2)),
                (Some(Value::Bool(true)), Value::Int(3)),
                (None, Value::Int(4)),
            ],
            &mut asked,
        )
        .unwrap();
        assert_eq!(
            (chosen.value.clone(), chosen.unmatched, chosen.by_rows),
            (Value::Int(3), 0, false)
        );
        assert_eq!(
            asked,
            [
                Part::Condition(0),
                Part::Condition(1),
                Part::Condition(2),
                Part::Value(2)
            ]
        );
        assert!(chosen.warning().is_none());
    }

    #[test]
    fn no_arm_matched_is_na_with_a_warning() {
        let chosen = run(
            &[(Some(Value::Bool(false)), Value::Int(1))],
            &mut Vec::new(),
        )
        .unwrap();
        assert_eq!((chosen.value.clone(), chosen.unmatched), (Value::Na, 1));
        let warning = chosen.warning().unwrap();
        assert_eq!(warning.code, "W0208");
        assert_eq!(
            warning.message,
            "no arm of the `case` matched, so its value is NA"
        );
    }

    #[test]
    fn a_column_turns_the_rest_row_wise() {
        let mut asked = Vec::new();
        let chosen = run(
            &[
                (Some(Value::Bool(false)), Value::Int(9)),
                (
                    Some(bools(&[Some(true), Some(false), None, Some(false)])),
                    Value::Int(1),
                ),
                (
                    Some(bools(&[Some(true), Some(true), Some(false), Some(false)])),
                    Value::Column(Column::floats([Some(0.5), Some(1.5), Some(2.5), Some(3.5)]).0),
                ),
            ],
            &mut asked,
        )
        .unwrap();
        // The first row has the first true arm, an Int, which joins the Floats.
        let Value::Column(column) = &chosen.value else {
            panic!("not a column");
        };
        assert_eq!(column.ty(), ColumnType::Float);
        let cells: Vec<Value> = (0..4).map(|r| lift::cell_value(column, r)).collect();
        assert_eq!(
            cells,
            [Value::Float(1.0), Value::Float(1.5), Value::Na, Value::Na]
        );
        assert_eq!((chosen.unmatched, chosen.by_rows), (2, true));
        assert_eq!(
            chosen.warning().unwrap().message,
            "2 rows matched no arm of the `case` and are NA"
        );
        // The value of the arm that was false for one value is never asked for.
        assert!(!asked.contains(&Part::Value(0)));
    }

    #[test]
    fn a_catch_all_supplies_the_rows_left_and_an_arm_with_none_is_not_evaluated() {
        let mut asked = Vec::new();
        let chosen = run(
            &[
                (
                    Some(bools(&[Some(true), Some(true)])),
                    Value::Str("a".into()),
                ),
                (
                    Some(bools(&[Some(true), Some(false)])),
                    Value::Str("b".into()),
                ),
                (None, Value::Str("c".into())),
            ],
            &mut asked,
        )
        .unwrap();
        assert_eq!(chosen.unmatched, 0);
        // Every row was taken by the first arm: no later part is asked for.
        assert_eq!(asked, [Part::Condition(0), Part::Value(0)]);
        let chosen = run(
            &[
                (
                    Some(bools(&[Some(true), Some(false)])),
                    Value::Str("a".into()),
                ),
                (None, Value::Str("c".into())),
            ],
            &mut Vec::new(),
        )
        .unwrap();
        let Value::Column(column) = &chosen.value else {
            panic!("not a column");
        };
        assert_eq!(lift::cell_value(column, 1), Value::Str("c".into()));
    }

    #[test]
    fn what_a_case_refuses() {
        let failed = |arms: &[(Option<Value>, Value)]| match run(arms, &mut Vec::new()) {
            Err(Failed::Trouble(trouble)) => *trouble,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            failed(&[(Some(Value::Int(1)), Value::Int(1))]),
            Trouble::Condition {
                arm: 0,
                value: Value::Int(1)
            }
        );
        assert!(matches!(
            failed(&[(Some(Value::Column(Column::ints([Some(1)]))), Value::Int(1))]),
            Trouble::Condition { arm: 0, .. }
        ));
        assert_eq!(
            failed(&[
                (Some(bools(&[Some(false), Some(true)])), Value::Int(1)),
                (Some(bools(&[Some(true)])), Value::Int(2)),
            ]),
            Trouble::Rows {
                part: Part::Condition(1),
                found: 1,
                rows: 2
            }
        );
        assert_eq!(
            failed(&[(
                Some(bools(&[Some(true), Some(true)])),
                Value::Column(Column::ints([Some(1)]))
            )]),
            Trouble::Rows {
                part: Part::Value(0),
                found: 1,
                rows: 2
            }
        );
        assert_eq!(
            failed(&[
                (Some(bools(&[Some(true), Some(false)])), Value::Int(1)),
                (None, Value::Str("a".into())),
            ]),
            Trouble::Mixed { start: 0 }
        );
        // An evaluation that fails is passed on as it is.
        let conditions = [true];
        let passed: Result<Chosen, Failed<&str>> = choose(&conditions, |_| Err("stopped"));
        assert!(matches!(passed, Err(Failed::Other("stopped"))));
    }
}
